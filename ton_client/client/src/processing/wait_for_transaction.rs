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
use ton_sdk::types::{TRANSACTIONS_TABLE_NAME, StringId};
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
    pub message_expiration_time: Option<u32>,
    /// Waiting options.
    pub processing_options: Option<ProcessingOptions>,
    /// Waiting state.
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
    let message_id_hex = hex::encode(&message_id);
    let address = message
        .dst()
        .ok_or(Error::message_has_not_destination_address())?;

    let stop_time = match params.message_expiration_time {
        Some(time) => time as u64,
        None => context.now_millis() + client.config().message_processing_timeout() as u64,
    };
    let add_timeout = client.config().message_processing_timeout() as u64;
    let mut transaction = Value::Null;
    let mut processing_state = params.processing_state;
    let mut network_retries = 0;
    let (
        network_retries_limit,
        network_retries_timeout,
        expiration_retries_limit,
        expiration_retries_timeout,
    ) = ProcessingOptions::resolve(&params.processing_options, &context);

    // Wait loop
    loop {
        // Fetch next shard block
        let now = context.now_millis();
        let timeout = std::cmp::max(stop_time, now) - now + add_timeout;
        if let Some(cb) = &params.callback {
            ProcessingEvent::WillFetchNextBlock {
                processing_state: processing_state.clone(),
                message_id: message_id_hex.clone(),
                message: params.message.clone(),
            }
            .emit(&context, cb)
        }
        let result = Block::wait_next_block(
            client,
            &StringId::from(&processing_state.last_checked_block_id),
            &address,
            Some((timeout / 1000) as u32),
        )
        .await;
        let block = match result {
            Err(err) => {
                if let Some(cb) = &params.callback {
                    ProcessingEvent::FetchNextBlockFailed {
                        processing_state: processing_state.clone(),
                        message_id: message_id_hex.clone(),
                        message: params.message.clone(),
                        error,
                    }
                    .emit(&context, cb)
                }
                if let Some(&SdkError::WaitForTimeout) = err.downcast_ref::<SdkError>() {
                    if infinite_wait {
                        continue;
                    }
                    return Err(Error::fetch_block_failed(
                        err,
                        &message_id_hex,
                        &processing_state,
                    ));
                } else {
                    if infinite_wait {
                        context.delay_ms(network_retries_timeout as u64).await;
                        continue;
                    }
                    return Err(Error::fetch_block_failed(
                        err,
                        &message_id_hex,
                        &processing_state,
                    ));
                }
            }
            Ok(block) => block,
        };

        processing_state.last_checked_block_id = block.id.to_string();

        // Find transaction in block
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
        if !transaction.is_null() {
            break;
        }

        // Check if time has been expired
        if block.gen_utime > stop_time {
            return Err(if expire.is_some() {
                Error::message_expired(&message_id_hex, &processing_state)
            } else {
                Error::transaction_wait_timeout(&message_id_hex, &processing_state)
            });
        }
    }

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
