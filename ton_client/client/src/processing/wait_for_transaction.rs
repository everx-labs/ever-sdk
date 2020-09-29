use crate::abi::ParamsOfEncodeMessage;
use crate::client::ClientContext;
use crate::encoding::base64_decode;
use crate::error::{ApiError, ApiResult};
use crate::processing::defaults::DEFAULT_NETWORK_RETRIES_LIMIT;
use crate::processing::internal::get_message_id;
use crate::processing::types::{
    CallbackParams, ProcessingEvent, ProcessingOptions, ProcessingState, TransactionResult,
};
use crate::processing::Error;
use serde_json::Value;
use std::sync::Arc;
use ton_block::MsgAddressInt;
use ton_sdk::node_client::MAX_TIMEOUT;
use ton_sdk::types::{StringId, TRANSACTIONS_TABLE_NAME};
use ton_sdk::{
    Block, BlockId, Contract, MessageId, MessageProcessingState, NodeClient, ReceivedTransaction,
    SdkError, SdkMessage, Transaction,
};

//--------------------------------------------------------------------------- wait_for_transaction

const MESSAGE_EXPIRED_CODE: i32 = 57;
const REPLAY_PROTECTION_CODE: i32 = 52;

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub struct ParamsOfWaitForTransaction {
    /// Message BOC. Encoded with `base64`.
    pub message: String,
    /// Message expiration time.
    /// Used only for messages with `expiration` replay protection.
    /// Must be the same value as it specified in `expire` ABI header
    /// of the message body.
    pub message_expiration_time: Option<u32>,
    /// Processing options.
    pub processing_options: Option<ProcessingOptions>,
    /// Processing state. As it received from `send_message`
    /// or 'Incomplete` variant of result of the `wait_for_transaction`.
    pub processing_state: ProcessingState,
    /// Processing callback.
    pub callback: Option<CallbackParams>,
}

#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug)]
pub enum ResultOfWaitForTransaction {
    Complete(TransactionResult),
    Incomplete(ProcessingState),
}

pub async fn wait_for_transaction(
    context: Arc<ClientContext>,
    params: ParamsOfWaitForTransaction,
) -> ApiResult<ResultOfWaitForTransaction> {
    let client = context.get_client()?;

    // Prepare to wait
    let message = Contract::deserialize_message(&base64_decode(&params.message)?)
        .map_err(|err| Error::invalid_message_boc(err))?;
    let message_id = get_message_id(&message)?;
    let address = message
        .dst()
        .ok_or(Error::message_has_not_destination_address())?;

    let expiration_time = match params.message_expiration_time {
        Some(time) => time as u64,
        None => context.now_millis() + client.config().message_processing_timeout() as u64,
    };
    let add_timeout = client.config().message_processing_timeout() as u64;
    let mut transaction = Value::Null;
    let mut processing_state = params.processing_state;
    let mut network_retries = 0;
    let now = context.now_millis();
    let timeout = std::cmp::max(expiration_time, now) - now + add_timeout;

    // Wait loop
    loop {
        match fetch_next_shard_block(
            &context,
            &params,
            &message_id,
            &address,
            timeout,
            &processing_state,
        )? {
            Ok(block) => {
                processing_state.last_checked_block_id = block_id;
                if let Some(transaction) = find_transaction_in_block(block)? {
                    return get_transaction_result();
                }
                if block.gen_utime > expiration_time {
                    return Err(if params.message_expiration_time.is_some() {
                        Error::message_expired(&message_id_hex, &processing_state)
                    } else {
                        Error::transaction_wait_timeout(&message_id_hex, &processing_state)
                    });
                }
            },
            Err(error) {
                return Ok(ResultOfWaitForTransaction::Incomplete {
                    processing_state: processing_state,
                    error,
                }))
            }

        }
    }
}

async fn fetch_next_shard_block(
    context: &Arc<ClientContext>,
    params: &ParamsOfWaitForTransaction,
    message_id: &str,
    address: &MsgAddressInt,
    timeout: u64,
    processing_state: &ProcessingState,
) -> ApiResult<Option<Block>> {
    let mut retries: i8 = 0;
    let current = StringId::from(&processing_state.last_checked_block_id);
    loop {
        if let Some(cb) = &params.callback {
            ProcessingEvent::WillFetchNextBlock {
                processing_state: processing_state.clone(),
                message_id: message_id.to_string(),
                message: params.message.clone(),
            }
            .emit(&context, cb)
        }
        match Block::wait_next_block(
            context.get_client()?,
            &current,
            &address,
            Some((timeout / 1000) as u32),
        )
        .await
        {
            Ok(block) => return Ok(Some(block)),
            Err(err) => {
                let error = Error::fetch_block_failed(err, &message_id, &processing_state);
                if let Some(cb) = &params.callback {
                    ProcessingEvent::FetchNextBlockFailed {
                        processing_state: processing_state.clone(),
                        message_id: message_id_hex.clone(),
                        message: params.message.clone(),
                        error: error.clone(),
                    }
                    .emit(&context, cb)
                }
                    if !params.processing_options.can_retry_network_error(context, &mut retries) {
                        return Err(error);
                    }
            }
        }
        context.delay_ms(network_retries_timeout as u64).await;

    }
}

fn find_transaction_in_block() {
    for block_msg in &block.in_msg_descr {
        if Some(message_id) == block_msg.msg_id.as_ref() {
            let tr_id = block_msg
                .transaction_id
                .clone()
                .ok_or(Error::invalid_block_received(
                    "No field `transaction_id` in block",
                    &message_id_hex,
                    &processing_state,
                ))?;

            transaction = client
                .wait_for(
                    TRANSACTIONS_TABLE_NAME,
                    &json!({
                        "id": { "eq": tr_id.to_string() }
                    })
                    .to_string(),
                    TRANSACTION_FIELDS_ORDINARY,
                    Some(MAX_TIMEOUT),
                )
                .await?;

            break;
        }
    }
}

fn get_transaction_result() {
    let parsed = serde_json::from_value::<Transaction>(transaction.clone())?;
    match parsed.compute.exit_code {
        Some(MESSAGE_EXPIRED_CODE) | Some(REPLAY_PROTECTION_CODE) => {
            Err(Error::message_expired(&message_id_hex, &processing_state))
        }
        _ => Ok(ResultOfWaitForTransaction::Complete(TransactionResult {
            transaction: Value::Null,
            out_messages: vec![],
            abi_return_value: None,
        })),
    }
}
