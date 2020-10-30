use crate::abi::{Abi, ParamsOfDecodeMessage};
use crate::client::ClientContext;
use crate::error::{ClientError, ClientResult};
use crate::processing::Error;
use crate::tvm::{AccountForExecutor, ParamsOfRunExecutor};
use super::fetching::fetch_account;
use std::sync::Arc;
use ton_block::{MsgAddressInt, Serializable};
use ton_sdk::{Block, MessageId};

pub(crate) fn get_message_id(message: &ton_block::Message) -> ClientResult<String> {
    let cells: ton_types::Cell = message
        .write_to_new_cell()
        .map_err(|err| Error::can_not_build_message_cell(err))?
        .into();
    let id: Vec<u8> = cells.repr_hash().as_slice()[..].into();
    Ok(hex::encode(&id))
}

/// Increments `retries` and returns `true` if `retries` hasn't reached `limit`.
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

pub(crate) async fn resolve_error(
    context: Arc<ClientContext>,
    address: &MsgAddressInt,
    message: String,
    mut original_error: ClientError,
) -> ClientResult<()> {
    let account = fetch_account(context.clone(), address, "boc").await?;

    let boc = account["boc"]
        .as_str()
        .ok_or(Error::invalid_data("Account doesn't contain 'boc'"))?
        .to_owned();

    let result = crate::tvm::run_executor(
        context,
        ParamsOfRunExecutor {
            abi: None,
            account: AccountForExecutor::Account { boc, unlimited_balance: None },
            execution_options: None,
            message,
            skip_transaction_check: None
        }
    ).await;

    match result {
        Err(mut err) => {
            err.data["original_error"] = serde_json::json!(original_error);
            Err(err)
        },
        Ok(_) => {
            original_error.data["disclaimer"] = "Local contract call succeded. Can not resolve extended error".into();
            Err(original_error)
        }
    }
}
