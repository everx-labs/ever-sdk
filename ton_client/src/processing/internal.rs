use super::ErrorCode;
use super::fetching::fetch_account;
use crate::abi::{Abi, ParamsOfDecodeMessage};
use crate::client::ClientContext;
use crate::error::{ClientError, ClientResult};
use crate::processing::Error;
use crate::tvm::{AccountForExecutor, ExecutionOptions, ParamsOfRunExecutor};
use std::sync::Arc;
use ton_block::MsgAddressInt;
use ton_sdk::{Block, MessageId};

/// Increments `retries` and returns `true` if `retries` hasn't reached `limit`.
pub(crate) fn can_retry_more(retries: u8, limit: i8) -> bool {
    limit < 0 || retries < limit as u8
}

pub fn can_retry_network_error(context: &Arc<ClientContext>, start: u64) -> bool {
    context.env.now_ms() < start + context.config.network.max_reconnect_timeout as u64
}

pub(crate) fn can_retry_expired_message(context: &Arc<ClientContext>, retries: u8) -> bool {
    can_retry_more(retries, context.config.network.message_retries_count)
}

pub fn find_transactions(
    block: &Block,
    message_id: &str,
    shard_block_id: &String,
) -> ClientResult<Vec<String>> {
    let mut ids = Vec::new();
    let msg_id: MessageId = message_id.into();
    for msg_descr in &block.in_msg_descr {
        if Some(&msg_id) == msg_descr.msg_id.as_ref() {
            ids.push(
                msg_descr
                    .transaction_id
                    .as_ref()
                    .ok_or(Error::invalid_block_received(
                        "No field `transaction_id` in block's `in_msg_descr`.",
                        message_id,
                        shard_block_id,
                    ))?
                    .to_string(),
            );
        }
    }
    Ok(ids)
}

pub(crate) async fn get_message_expiration_time(
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
                allow_partial: false,
            },
        )
        .await
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

#[derive(Deserialize)]
struct Account {
    boc: String,
    last_paid: Option<u32>,
}

async fn get_local_error(
    context: Arc<ClientContext>,
    address: &MsgAddressInt,
    message: String,
    time: u32,
    show_tips_on_error: bool,
) -> ClientResult<String> {
    let account = fetch_account(context.clone(), address, "boc last_paid").await?;

    let account: Account = serde_json::from_value(account)
        .map_err(|err| Error::invalid_data(format!("Can not parse account for error resolving: {}", err)))?;

    if let Some(last_paid) = account.last_paid {
        if last_paid > time {
            return Ok("Can not resolve error due to modified account state".to_owned());
        }
    }

    crate::tvm::run_executor_internal(
        context,
        ParamsOfRunExecutor {
            abi: None,
            account: AccountForExecutor::Account {
                boc: account.boc,
                unlimited_balance: None,
            },
            execution_options: Some(ExecutionOptions {
                block_time: Some(time),
                ..Default::default()
            }),
            message,
            ..Default::default()
        },
        show_tips_on_error,
    )
    .await
    .map(|_| "Local contract call emulation was successful".to_owned())
}

pub(crate) async fn resolve_error(
    context: Arc<ClientContext>,
    address: &MsgAddressInt,
    message: String,
    mut original_error: ClientError,
    time: u32,
    without_transaction: bool,
) -> ClientResult<()> {
    let result = get_local_error(context, address, message, time, without_transaction).await;

    match result {
        Err(err) => {
            const EXIT_CODE_FIELD: &str = "exit_code";
            const EXIT_ARG_FIELD: &str = "exit_arg";
            const CONTRACT_ERROR_FIELD: &str = "contract_error";

            let exit_code = original_error.data[EXIT_CODE_FIELD].as_i64();
            let local_exit_code = err.data[EXIT_CODE_FIELD].as_i64();

            if !without_transaction && exit_code != local_exit_code {
                return Err(original_error);
            }

            if without_transaction {
                original_error.data["local_error"] =
                    serde_json::to_value(&err).map_err(crate::client::Error::cannot_serialize_result)?;
            } else {
                original_error.data[EXIT_ARG_FIELD] = err.data[EXIT_ARG_FIELD].clone();
                original_error.data[CONTRACT_ERROR_FIELD] = err.data[CONTRACT_ERROR_FIELD].clone();
            }

            match original_error.message.find("\nTip:") {
                Some(insert_position) => {
                    original_error.message = format!(
                        "{}.\nPossible reason: {}.{}",
                        &original_error.message[..insert_position].trim_end().trim_end_matches("."),
                        remove_exit_code(&exit_code, err.message.trim_end_matches(".")),
                        &original_error.message[insert_position..],
                    )
                },
                None => original_error.message = format!(
                    "{}.\nPossible reason: {}",
                    original_error.message.trim_end_matches("."),
                    remove_exit_code(&exit_code, &err.message),
                )
            }

            Err(original_error)
        }
        Ok(message) => {
            original_error.message = format!(
                "{}. {}. Possible reason: message has not been delivered",
                original_error.message.trim_end_matches("."),
                message,
            );
            if original_error.code == ErrorCode::MessageExpired as u32 {
                original_error.message = format!(
                    "{}. Try to send it again",
                    original_error.message,
                );
            }
            Err(original_error)
        }
    }
}

/// Removes exit code from internal error only if it matches exit code of original error
fn remove_exit_code(exit_code: &Option<i64>, internal_error: &str) -> String {
    if let Some(exit_code) = exit_code {
        regex::Regex::new(&format!(r#"(?i)([,\.]\s*)?exit\s+code(:\s*|\s+){}"#, exit_code)).unwrap()
            .replace(internal_error, "").to_string()
    } else {
        internal_error.to_string()
    }
}
