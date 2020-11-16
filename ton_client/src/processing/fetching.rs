use crate::abi::Abi;
use crate::boc::internal::deserialize_object_from_base64;
use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::net::{
    wait_for_collection, ParamsOfWaitForCollection, MAX_TIMEOUT, TRANSACTIONS_TABLE_NAME,
};
use crate::processing::blocks_walking::wait_next_block;
use crate::processing::internal::can_retry_network_error;
use crate::processing::parsing::{decode_output, parse_transaction_boc};
use crate::processing::{
    Error, ParamsOfWaitForTransaction, ProcessingEvent, ResultOfProcessMessage,
};
use crate::tvm::check_transaction::calc_transaction_fees;
use serde_json::Value;
use std::sync::Arc;
use ton_block::MsgAddressInt;
use ton_sdk::Block;

pub async fn fetch_next_shard_block<F: futures::Future<Output = ()> + Send + Sync>(
    context: &Arc<ClientContext>,
    params: &ParamsOfWaitForTransaction,
    address: &MsgAddressInt,
    block_id: &str,
    message_id: &str,
    timeout: u32,
    callback: impl Fn(ProcessingEvent) -> F + Send + Sync,
) -> ClientResult<Block> {
    let mut retries: u8 = 0;
    let network_retries_timeout = context.config.network.network_retries_count;
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

                // If network retries limit has reached, return error
                if !can_retry_network_error(context, retries) {
                    return Err(error);
                }
            }
        }

        // Perform delay before retry
        context
            .env
            .set_timer(network_retries_timeout as u64)
            .await?;
        retries = retries.checked_add(1).unwrap_or(retries);
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
                collection: TRANSACTIONS_TABLE_NAME.into(),
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
    transaction_id: &str,
    abi: &Option<Abi>,
    address: MsgAddressInt,
    expiration_time: u32,
    block_time: u32,
) -> ClientResult<ResultOfProcessMessage> {
    let transaction_boc =
        fetch_transaction_boc(context, transaction_id, message_id, shard_block_id).await?;
    let context_copy = context.clone();
    let get_contract_info = || async move {
        let balance = fetch_contract_balance(context_copy, &address).await?;
        Ok((address, balance))
    };
    let transaction_object = deserialize_object_from_base64(&transaction_boc.boc, "transaction")?;

    let fees = calc_transaction_fees(&transaction_object.object, true, false, get_contract_info)
        .await
        .map_err(|err| {
            if err.code == crate::tvm::ErrorCode::ContractExecutionError as u32
                && (err.data["exit_code"] == crate::tvm::StdContractError::ReplayProtection as i32
                    || err.data["exit_code"]
                        == crate::tvm::StdContractError::ExtMessageExpired as i32)
            {
                Error::message_expired(&message_id, shard_block_id, expiration_time, block_time)
            } else {
                err
            }
        })?;

    let (transaction, out_messages) = parse_transaction_boc(context.clone(), transaction_boc)?;
    let abi_decoded = if let Some(abi) = abi {
        Some(decode_output(context, abi, out_messages.clone())?)
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
    let mut retries: u8 = 0;
    let network_retries_timeout = context.config.network.network_retries_count;

    // Network retries loop
    loop {
        match TransactionBoc::fetch_value(context, transaction_id).await {
            Ok(value) => {
                return Ok(TransactionBoc::from(value, message_id, shard_block_id)?);
            }
            Err(error) => {
                // If network retries limit has reached, return error
                if !can_retry_network_error(context, retries) {
                    return Err(error);
                }
            }
        }

        // Perform delay before retry
        context
            .env
            .set_timer(network_retries_timeout as u64)
            .await?;
        retries = retries.checked_add(1).unwrap_or(retries);
    }
}
