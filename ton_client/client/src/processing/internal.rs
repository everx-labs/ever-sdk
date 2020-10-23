use crate::abi::{Abi, ParamsOfDecodeMessage};
use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::processing::Error;
use serde_json::Value;
use std::sync::Arc;
use ton_block::Serializable;
use ton_sdk::{Block, MessageId};

pub(crate) fn get_message_id(message: &ton_block::Message) -> ClientResult<String> {
    let cells: ton_types::Cell = message
        .write_to_new_cell()
        .map_err(|err| Error::can_not_build_message_cell(err))?
        .into();
    let id: Vec<u8> = cells.repr_hash().as_slice()[..].into();
    Ok(hex::encode(&id))
}

/// Increments `retries` and returns `true` if `retries` isn't reach `limit`.
pub(crate) fn can_retry_more(retries: u8, limit: i8) -> bool {
    limit < 0 || retries <= limit as u8
}

pub fn can_retry_network_error(context: &Arc<ClientContext>, retries: u8) -> bool {
    can_retry_more(retries, context.config.network.network_retries_count)
}

pub(crate) fn can_retry_expired_message(context: &Arc<ClientContext>, retries: u8) -> bool {
    can_retry_more(retries, context.config.network.message_retries_count)
}

pub fn find_transaction(
    block: &Block,
    message_id: &str,
    shard_block_id: &String,
) -> ClientResult<Option<String>> {
    let msg_id: MessageId = message_id.into();
    for msg_descr in &block.in_msg_descr {
        if Some(&msg_id) == msg_descr.msg_id.as_ref() {
            return Ok(Some(
                msg_descr
                    .transaction_id
                    .as_ref()
                    .ok_or(Error::invalid_block_received(
                        "No field `transaction_id` in block's `in_msg_descr`.",
                        message_id,
                        shard_block_id,
                    ))?
                    .to_string(),
            ));
        }
    }
    Ok(None)
}

#[derive(Deserialize)]
struct ComputePhase {
    exit_code: i32,
}

#[derive(Deserialize)]
struct Transaction {
    compute: ComputePhase,
}

pub(crate) fn get_exit_code(
    parsed_transaction: &Value,
    shard_block_id: &String,
    message_id: &str,
) -> ClientResult<i32> {
    Ok(
        serde_json::from_value::<Transaction>(parsed_transaction.clone())
            .map_err(|err| {
                Error::fetch_transaction_result_failed(
                    format!("Transaction can't be parsed: {}", err),
                    message_id,
                    shard_block_id,
                )
            })?
            .compute
            .exit_code,
    )
}

pub(crate) fn get_message_expiration_time(
    context: Arc<ClientContext>,
    abi: Option<&Abi>,
    message: &str,
) -> ClientResult<Option<u64>> {
    let header = match abi {
        Some(abi) => crate::abi::decode_message(
            context.clone(),
            ParamsOfDecodeMessage {
                abi: abi.clone(),
                message: message.to_string(),
            },
        )
        .map(|x| x.header)
        .unwrap_or_default(),
        None => None,
    };
    let time = header
        .as_ref()
        .map_or(None, |x| x.expire)
        .map(|x| x as u64 * 1000);
    Ok(time)
}
