use crate::abi::Abi;
use crate::boc::internal::deserialize_object_from_base64;
use crate::client::ClientContext;
use crate::error::{ClientResult, AddNetworkUrl};
use crate::net::{
    wait_for_collection, ParamsOfWaitForCollection, MAX_TIMEOUT, TRANSACTIONS_COLLECTION,
};
use crate::processing::blocks_walking::wait_next_block;
use crate::processing::internal::{can_retry_network_error, resolve_error};
use crate::processing::parsing::{decode_output, parse_transaction_boc};
use crate::processing::{
    Error, ParamsOfWaitForTransaction, ProcessingEvent, ResultOfProcessMessage,
};
use crate::tvm::check_transaction::{calc_transaction_fees, extract_error};
use serde_json::Value;
use std::convert::TryFrom;
use std::sync::Arc;
use ton_block::MsgAddressInt;
use ton_sdk::Block;

pub async fn fetch_next_shard_block<F: futures::Future<Output = ()> + Send>(
    context: &Arc<ClientContext>,
    params: &ParamsOfWaitForTransaction,
    address: &MsgAddressInt,
    block_id: &str,
    message_id: &str,
    timeout: u32,
    callback: impl Fn(ProcessingEvent) -> F + Send + Sync,
) -> ClientResult<Block> {
    let start = context.env.now_ms();

    // Network retries loop
    loop {
        // Notify app about fetching next block
        if params.send_events {
            callback(ProcessingEvent::WillFetchNextBlock {
                shard_block_id: block_id.to_string(),
                message_id: message_id.to_string(),
                message: params.message.clone(),
            })
            .await;
        }

        // Fetch next block
        match wait_next_block(context, block_id.into(), &address, Some(timeout)).await {
            Ok(block) => return Ok(block),
            Err(err) => {
                let is_retryable_error = crate::client::Error::is_network_error(&err) ||
                    err.code == crate::net::ErrorCode::WaitForTimeout as u32;
                let error = Error::fetch_block_failed(err, &message_id, &block_id.to_string());

                // Notify app about error
                if params.send_events {
                    callback(ProcessingEvent::FetchNextBlockFailed {
                        shard_block_id: block_id.to_string(),
                        message_id: message_id.to_string(),
                        message: params.message.clone(),
                        error: error.clone(),
                    })
                    .await;
                }

                // If network retries timeout has reached, return error
                if !is_retryable_error || !can_retry_network_error(context, start)
                {
                    return Err(error);
                }
            }
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct MessageBoc {
    pub boc: String,
}

#[derive(Deserialize)]
pub(crate) struct TransactionBoc {
    pub boc: String,
    pub out_messages: Vec<MessageBoc>,
}

impl TransactionBoc {
    async fn fetch_value(
        context: &Arc<ClientContext>,
        transaction_id: &str,
    ) -> ClientResult<Value> {
        Ok(wait_for_collection(
            context.clone(),
            ParamsOfWaitForCollection {
                collection: TRANSACTIONS_COLLECTION.into(),
                filter: Some(json!({
                    "id": { "eq": transaction_id.to_string() }
                })),
                result: "boc out_messages { boc }".into(),
                timeout: Some(MAX_TIMEOUT),
            },
        )
        .await?
        .result)
    }

    fn from(value: Value, message_id: &str, shard_block_id: &String) -> ClientResult<Self> {
        serde_json::from_value::<TransactionBoc>(value).map_err(|err| {
            Error::fetch_transaction_result_failed(
                format!("Transaction can't be parsed: {}", err),
                message_id,
                shard_block_id,
            )
        })
    }
}

pub(crate) async fn fetch_account(
    context: Arc<ClientContext>,
    address: &MsgAddressInt,
    result: &str,
) -> ClientResult<Value> {
    let mut result = crate::net::query_collection(
        context,
        crate::net::ParamsOfQueryCollection {
            collection: "accounts".to_owned(),
            filter: Some(serde_json::json!({
                "id": { "eq": address.to_string() }
            })),
            limit: None,
            order: None,
            result: result.to_owned(),
        },
    )
    .await?;

    result
        .result
        .pop()
        .ok_or(crate::tvm::Error::account_missing(address))
}

#[derive(Deserialize)]
struct AccountBalance {
    #[serde(with = "ton_sdk::json_helper::uint")]
    balance: u64,
}

async fn fetch_contract_balance(
    context: Arc<ClientContext>,
    address: &MsgAddressInt,
) -> ClientResult<u64> {
    let account = fetch_account(context, address, "balance").await?;

    let balance: AccountBalance = serde_json::from_value(account)
        .map_err(|err| Error::invalid_data(format!("can not parse account balance: {}", err)))?;

    Ok(balance.balance)
}

pub async fn fetch_transaction_result(
    context: &Arc<ClientContext>,
    shard_block_id: &String,
    message_id: &str,
    message: &str,
    transaction_id: &str,
    abi: &Option<Abi>,
    address: MsgAddressInt,
    expiration_time: u32,
    block_time: u32,
) -> ClientResult<ResultOfProcessMessage> {
    let transaction_boc =
        fetch_transaction_boc(context, transaction_id, message_id, shard_block_id).await?;
    let context_copy = context.clone();
    let address_copy = address.clone();
    let get_contract_info = || async move {
        let balance = fetch_contract_balance(context_copy, &address_copy).await?;
        Ok((address_copy, balance))
    };
    let transaction_object = deserialize_object_from_base64(&transaction_boc.boc, "transaction")?;

    let transaction = ton_sdk::Transaction::try_from(&transaction_object.object)
        .map_err(|err| crate::tvm::Error::can_not_read_transaction(err))?;

    let local_result = if transaction.is_aborted() {
        let error = match extract_error(&transaction, get_contract_info.clone(), true).await {
            Err(err) => err,
            Ok(_) => crate::tvm::Error::transaction_aborted(),
        };

        Some(resolve_error(
            Arc::clone(context),
            &address,
            message.to_string(),
            error,
            expiration_time - 1,
        )
            .await
            .add_network_url_from_context(&context)
            .await
            .map_err(|mut error| {
                error.data["transaction_id"] = transaction.id().to_string().into();
                error
            }))
    } else {
        None
    };

    let fees = calc_transaction_fees(&transaction, true, false, get_contract_info, true)
        .await
        .map_err(|err| {
            if err.code == crate::tvm::ErrorCode::ContractExecutionError as u32
                && (err.data["exit_code"] == crate::tvm::StdContractError::ReplayProtection as i32
                    || err.data["exit_code"]
                        == crate::tvm::StdContractError::ExtMessageExpired as i32)
            {
                Error::message_expired(&message_id, shard_block_id, expiration_time, block_time, &address)
            } else {
                if let Some(local_result) = local_result {
                    if let Err(local_error) = local_result {
                        if local_error.code == err.code {
                            return local_error;
                        }
                    }
                }
                err
            }
        })?;

    let (transaction, out_messages) = parse_transaction_boc(context.clone(), transaction_boc).await?;
    let abi_decoded = if let Some(abi) = abi {
        Some(decode_output(context, abi, out_messages.clone()).await?)
    } else {
        None
    };

    Ok(ResultOfProcessMessage {
        transaction,
        out_messages,
        decoded: abi_decoded,
        fees,
    })
}

async fn fetch_transaction_boc(
    context: &Arc<ClientContext>,
    transaction_id: &str,
    message_id: &str,
    shard_block_id: &String,
) -> ClientResult<TransactionBoc> {
    let start = context.env.now_ms();

    // Network retries loop
    loop {
        match TransactionBoc::fetch_value(context, transaction_id).await {
            Ok(value) => {
                return Ok(TransactionBoc::from(value, message_id, shard_block_id)?);
            }
            Err(error) => {
                // If network retries timeout has reached, return error
                if !crate::client::Error::is_network_error(&error) ||
                    !can_retry_network_error(context, start)
                {
                    return Err(error);
                }
            }
        }
    }
}
