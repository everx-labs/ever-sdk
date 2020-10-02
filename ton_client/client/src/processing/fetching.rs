use crate::abi::Abi;
use crate::client::ClientContext;
use crate::error::ApiResult;
use crate::net::{wait_for_collection, ParamsOfWaitForCollection, MAX_TIMEOUT};
use crate::processing::blocks_walking::wait_next_block;
use crate::processing::internal::{
    can_retry_network_error, get_exit_code, resolve_network_retries_timeout,
};
use crate::processing::parsing::{decode_abi_output, parse_transaction_boc};
use crate::processing::types::TvmExitCode;
use crate::processing::{
    Error, ParamsOfWaitForTransaction, ProcessingEvent, ProcessingState, TransactionOutput,
};
use serde_json::Value;
use std::sync::Arc;
use ton_block::MsgAddressInt;
use ton_sdk::types::TRANSACTIONS_TABLE_NAME;
use ton_sdk::Block;

pub async fn fetch_next_shard_block(
    context: &Arc<ClientContext>,
    params: &ParamsOfWaitForTransaction,
    address: &MsgAddressInt,
    block_id: &str,
    message_id: &str,
    timeout: u32,
) -> ApiResult<Block> {
    let mut retries: u8 = 0;
    let network_retries_timeout = resolve_network_retries_timeout(context);
    let block_id = block_id.to_string().into();
    // Network retries loop
    loop {
        // Notify app about fetching next block
        if let Some(cb) = &params.callback {
            ProcessingEvent::WillFetchNextBlock {
                block_id: block_id.to_string(),
                message_id: message_id.to_string(),
                message: params.message.clone(),
            }
            .emit(&context, cb);
        }

        // Fetch next block
        match wait_next_block(context, &block_id, &address, Some(timeout)).await {
            Ok(block) => return Ok(block),
            Err(err) => {
                let error = Error::fetch_block_failed(err, &message_id, &processing_state);

                // Notify app about error
                if let Some(cb) = &params.callback {
                    ProcessingEvent::FetchNextBlockFailed {
                        block_id: block_id.to_string(),
                        message_id: message_id.to_string(),
                        message: params.message.clone(),
                        error: error.clone(),
                    }
                    .emit(&context, cb)
                }

                // If network retries limit has reached, return error
                if !can_retry_network_error(context, &mut retries) {
                    return Err(error);
                }
            }
        }

        // Perform delay before retry
        context.env.set_timer(network_retries_timeout as u64).await;
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
    async fn fetch_value(context: &Arc<ClientContext>, transaction_id: &str) -> ApiResult<Value> {
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

    fn from(value: Value, message_id: &str, processing_state: &ProcessingState) -> ApiResult<Self> {
        serde_json::from_value::<TransactionBoc>(value).map_err(|err| {
            Error::fetch_transaction_result_failed(
                format!("Transaction can't be parsed: {}", err),
                message_id,
                processing_state,
            )
        })
    }
}

pub async fn fetch_transaction_result(
    context: &Arc<ClientContext>,
    params: &ParamsOfWaitForTransaction,
    processing_state: &ProcessingState,
    message_id: &str,
    transaction_id: &str,
    abi: &Option<Abi>,
) -> ApiResult<TransactionOutput> {
    let transaction_boc =
        fetch_transaction_boc(context, transaction_id, message_id, processing_state).await?;
    let (transaction, out_messages) = parse_transaction_boc(context.clone(), &transaction_boc)?;
    let abi_decoded = if let Some(abi) = abi {
        Some(decode_abi_output(context, abi, &out_messages)?)
    } else {
        None
    };
    let exit_code = get_exit_code(&transaction, processing_state, message_id)?;

    if exit_code == TvmExitCode::MessageExpired as i32
        || exit_code == TvmExitCode::ReplayProtection as i32
    {
        Err(Error::message_expired(&message_id, &processing_state))
    } else {
        let result = TransactionOutput {
            transaction,
            out_messages,
            abi_decoded,
        };
        if let Some(cb) = &params.callback {
            ProcessingEvent::TransactionReceived {
                message_id: message_id.to_string(),
                message: params.message.clone(),
                result: result.clone(),
            }
            .emit(&context, cb);
        }
        Ok(result)
    }
}

async fn fetch_transaction_boc(
    context: &Arc<ClientContext>,
    transaction_id: &str,
    message_id: &str,
    processing_state: &ProcessingState,
) -> ApiResult<TransactionBoc> {
    let mut retries: u8 = 0;
    let network_retries_timeout = resolve_network_retries_timeout(context);

    // Network retries loop
    loop {
        match TransactionBoc::fetch_value(context, transaction_id).await {
            Ok(value) => {
                return Ok(TransactionBoc::from(value, message_id, processing_state)?);
            }
            Err(error) => {
                // If network retries limit has reached, return error
                if !can_retry_network_error(context, &mut retries) {
                    return Err(error);
                }
            }
        }

        // Perform delay before retry
        context.env.set_timer(network_retries_timeout as u64).await;
    }
}
