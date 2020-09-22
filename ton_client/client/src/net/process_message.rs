use crate::abi::ParamsOfEncodeMessage;
use crate::client::ClientContext;
use crate::encoding::base64_decode;
use crate::error::{ApiError, ApiResult};
use crate::net::Error;
use serde_json::Value;
use std::sync::Arc;
use ton_block::MsgAddressInt;
use ton_sdk::node_client::MAX_TIMEOUT;
use ton_sdk::types::TRANSACTIONS_TABLE_NAME;
use ton_sdk::{
    Block, BlockId, Contract, MessageId, MessageProcessingState, NodeClient, ReceivedTransaction,
    SdkError, SdkMessage, Transaction,
};

#[derive(Serialize, Deserialize, TypeInfo, Debug, Clone)]
pub struct CallbackParams {
    /// Callback ID.
    pub id: u32,
    /// Automatically unregister callback after process have been finished.
    pub unregister: bool,
}

#[derive(Serialize, Deserialize, TypeInfo, Debug, Clone)]
pub enum MessageProcessingEventType {
    /// Reports that message BOC will be encoded.
    /// Event occurs only for message source `AbiEncoding`.
    /// Event can occurs more than one time for messages with `expiration` replay protection
    /// in case of retries.
    EncodeMessage,
    /// Reports that account related block will be fetched from network.
    /// Event occurs only for message with `expiration` replay protection.
    /// Fetched block will be used later in waiting phase.
    FetchBlock,
    /// Reports that the starting block can't be fetched due to error.
    /// Messaged processing has finished.
    FetchBlockFailed,
    /// Reports that the message will be sent to the network.
    SendMessage,
    /// Reports that the message can't be sent due to network error.
    /// Processing will be continued at waiting phase.
    SendMessageFailed,
    /// Reports that next account related block will be fetched from network.
    /// Event occurs only for messages with `expiration` replay protection.
    /// Event can occurs more than one time due to block walking procedure.
    WaitFetchBlock,
    /// Reports that the next block can't be fetched due to error.
    /// Event occurs only for messages with `expiration` replay protection.
    /// Processing will be continued after network resuming timeout.
    WaitFetchBlockFailed,
    /// Reports that the message was expired.
    /// Event occurs for messages with `expiration` replay protection.
    /// Processing will be continued after expiration retries timeout
    /// at phase of encoding message.
    MessageExpired,
    /// Reports that the processing starts listening for a transaction.
    /// Event occurs for message without `expiration` replay protection.
    /// Processing will be continued after network resuming timeout.
    WaitTransaction,
    /// Reports that the transaction listening failed due to timeout or network error.
    /// Event occurs for message without `expiration` replay protection.
    /// Processing has finished with error.
    WaitTransactionFailed,
    /// Reports that the transaction received.
    /// Processing has finished.
    TransactionReceived,
}

#[derive(Serialize, Deserialize, TypeInfo, Debug, Clone)]
pub struct MessageProcessingEvent {
    /// Event type.
    event_type: MessageProcessingEventType,
    /// Transaction waiting state.
    transaction_waiting_state: TransactionWaitingState,
    /// Error describing reason of the failure.
    /// Presented in events that occurs when message processing
    /// encounters an error which is not fatal.
    /// Message processing will retry failed operation after this event.
    error: Option<ApiError>,
    /// Cancellation token. Application can use this token to cancel message processing using
    /// `client.cancel_operation` method.
    cancellation_token: u32,
}

impl MessageProcessingEvent {
    fn new(
        event_type: MessageProcessingEventType,
        transaction_waiting_state: TransactionWaitingState,
        error: Option<ApiError>,
        cancellation_token: u32,
    ) -> Self {
        Self {
            event_type,
            transaction_waiting_state,
            error,
            cancellation_token,
        }
    }
}

pub struct MessageMonitoringOptions {
    /// `true` if message processing must monitor network until the transaction appears.
    /// `false` if message processing must just send message to network.
    pub transaction_required: bool,
    /// Limit the retries count for expired messages.
    pub expiration_retries_limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExpirationWaitingState {
    /// The last shard block received before the message was sent
    /// or the last shard block checked for the resulting transaction
    /// after the message was sent.
    last_checked_block_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TransactionWaitingState {
    /// The waiting state for messages with `expiration` replay protection.
    expiration: Option<ExpirationWaitingState>,
    /// The time when the message was sent.
    message_sending_time: u32,
}

//----------------------------------------------------------------------------------- send_message

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub struct ParamsOfSendMessage {
    /// Message BOC.
    pub message: String,
    /// Message expiration time.
    /// Used only for messages with `expiration` replay protection.
    pub message_expiration_time: Option<u32>,
    /// Processing callback.
    pub callback: Option<CallbackParams>,
}

#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug)]
pub struct ResultOfSendMessage {
    pub transaction_waiting_state: TransactionWaitingState,
}

pub async fn send_message(
    context: Arc<ClientContext>,
    params: ParamsOfSendMessage,
) -> ApiResult<ResultOfSendMessage> {
    let now = context.now();
    if let Some(message_expiration_time) = params.message_expiration_time {
        if message_expiration_time <= now {
            return Err(Error::message_already_expired());
        }
    }

    let boc = base64_decode(&params.message)?;
    let message = Contract::deserialize_message(&boc)?;
    let id = get_message_id(&message)?;
    let address = message
        .dst()
        .ok_or(Error::message_has_not_destination_address())?;
    let client = context.get_client()?;
    let mut expiration = Option::<ExpirationWaitingState>::None;
    if params.message_expiration_time.is_some() {
        emit_event(&context, &params.callback, || {
            MessageProcessingEvent::FetchBlock
        });
        expiration = Some(ExpirationWaitingState {
            last_checked_block_id: Block::find_last_shard_block(client, &address).await?.into(),
        });
    }
    let mut transaction_waiting_state = TransactionWaitingState {
        expiration,
        message_sending_time: context.now(),
    };
    emit_event(&context, &params.callback, || {
        MessageProcessingEvent::SendMessage(transaction_waiting_state.clone())
    });
    client.send_message(&id, &boc).await?;
    Ok(ResultOfSendMessage {
        transaction_waiting_state,
    })
}

fn get_message_id(message: &TvmMessage) -> ApiResult<Vec<u8>> {
    let cells = message.write_to_new_cell()?.into();
    Ok(cells.repr_hash().as_slice()[..].into())
}

//--------------------------------------------------------------------------- wait_for_transaction

const MESSAGE_EXPIRED_CODE: i32 = 57;
const REPLAY_PROTECTION_CODE: i32 = 52;

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub struct ParamsOfWaitForTransaction {
    /// Message BOC.
    pub message: String,
    /// Message expiration time.
    /// Used only for messages with `expiration` replay protection.
    pub message_expiration_time: Option<u32>,
    /// Message expiration time.
    /// Used only for messages with `expiration` replay protection.
    pub transaction_waiting_state: TransactionWaitingState,
    /// Processing callback.
    pub callback: Option<CallbackParams>,
}

#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug)]
pub enum ResultOfWaitForTransaction {
    Complete(ReceivedTransaction),
    Incomplete(TransactionWaitingState, ApiError),
}


pub async fn wait_for_transaction(
    context: Arc<ClientContext>,
    params: ParamsOfWaitForTransaction,
) -> ApiResult<ResultOfWaitForTransaction> {
    let client = context.get_client()?;
    let stop_time = match params.message_expiration_time {
        Some(expire) => expire,
        None => context.now() + client.config().message_processing_timeout() / 1000,
    };

    let mut transaction = Value::Null;
    let add_timeout = client.config().message_processing_timeout();
    loop {
        let now = context.now();
        let timeout = std::cmp::max(stop_time, now) - now + add_timeout;
        let result = Block::wait_next_block(
            client,
            &processing_context.last_block_id,
            &address,
            Some(timeout),
        )
        .await;
        let block = match result {
            Err(err) => {
                log::debug!("wait_next_block error {}", err);
                if let Some(&SdkError::WaitForTimeout) = err.downcast_ref::<SdkError>() {
                    if infinite_wait {
                        log::warn!(
                            "Block awaiting timeout. Trying again. Current block {}",
                            processing_context.last_block_id
                        );
                        continue;
                    } else {
                        fail!(SdkError::NetworkSilent {
                            msg_id: message_id.clone(),
                            block_id: state.last_block_id.clone(),
                            timeout,
                            state
                        });
                    }
                } else if let Some(GraphiteError::NetworkError(_)) =
                    err.downcast_ref::<GraphiteError>()
                {
                    if infinite_wait {
                        log::warn!(
                            "Network error while awaiting next block for {}. Trying again.\n{}",
                            processing_context.last_block_id,
                            err
                        );
                        futures_timer::Delay::new(std::time::Duration::from_secs(1)).await;
                        continue;
                    } else {
                        fail!(SdkError::ResumableNetworkError { state, error: err });
                    }
                } else {
                    fail!(err);
                }
            }
            Ok(block) => block,
        };

        processing_context.last_block_id = block.id;

        for block_msg in &block.in_msg_descr {
            if Some(message_id) == block_msg.msg_id.as_ref() {
                let tr_id = block_msg
                    .transaction_id
                    .clone()
                    .ok_or(SdkError::InvalidData {
                        msg: "No field `transaction_id` in block".to_owned(),
                    })?;

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

        if block.gen_utime > stop_time {
            if expire.is_some() {
                fail!(SdkError::MessageExpired {
                    msg_id: message_id.clone(),
                    sending_time: state.sending_time,
                    expire: stop_time,
                    block_time: block.gen_utime,
                    block_id: state.last_block_id
                });
            } else {
                fail!(SdkError::TransactionWaitTimeout {
                    msg_id: message_id.clone(),
                    sending_time: state.sending_time,
                    timeout,
                    state
                });
            }
        }
    }

    let parsed = serde_json::from_value::<Transaction>(transaction.clone())?;
    if parsed.compute.exit_code == Some(Self::MESSAGE_EXPIRED_CODE)
        || parsed.compute.exit_code == Some(Self::REPLAY_PROTECTION_CODE)
    {
        Err(SdkError::MessageExpired {
            msg_id: message_id.clone(),
            sending_time: processing_context.sending_time,
            expire: expire.unwrap_or(0),
            block_time: parsed.now,
            block_id: transaction["block_id"].as_str().unwrap_or("null").into(),
        }
        .into())
    } else {
        Ok(ReceivedTransaction {
            parsed,
            value: transaction,
        })
    }
}

//-------------------------------------------------------------------------------- process_message

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub enum MessageSource {
    Message(String),
    AbiEncoding(ParamsOfEncodeMessage),
}

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub struct ParamsOfProcessMessage {
    /// Message source.
    pub message: MessageSource,
    /// Processing callback.
    pub callback: Option<CallbackParams>,
}

#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug)]
pub struct ResultOfProcessMessage {
    pub transaction: Option<Value>,
}

fn emit_event(
    context: &Arc<ClientContext>,
    callback: &Option<CallbackParams>,
    event: fn() -> MessageProcessingEvent,
) {
    if let Some(callback) = callback {
        let _ = context.send_callback_result(callback.id.clone(), event());
    }
}

fn ensure_message(
    context: &Arc<ClientContext>,
    source: &MessageSource,
    retry_count: u32,
    callback: &Option<CallbackParams>,
) -> ApiResult<(String, Option<u32>)> {
    Ok(match source {
        MessageSource::Message(boc) => (boc.clone(), None),
        MessageSource::AbiEncoding(encode_params) => {
            emit_event(context, callback, || MessageProcessingEvent::EncodeMessage);
            let encoded = crate::abi::encode_message(context.clone(), encode_params.clone())?;
            (encoded.message, None)
        }
    })
}

pub async fn process_message(
    context: Arc<ClientContext>,
    params: ParamsOfProcessMessage,
) -> ApiResult<ResultOfProcessMessage> {
    let mut retry_count = 0;
    loop {
        let (message, expiration_time) =
            ensure_message(&context, &params.message, retry_count, &params.callback)?;
        let transaction_waiting_state = send_message(
            context.clone(),
            ParamsOfSendMessage {
                message: message.clone(),
                message_expiration_time: None,
                callback: params.callback.clone(),
            },
        )
        .await?
        .transaction_waiting_state;

        let result = wait_for_transaction(
            context.clone(),
            ParamsOfWaitForTransaction {
                message: message.clone(),
                message_expiration_time: expiration_time,
                callback: params.callback.clone(),
                transaction_waiting_state,
            },
        )
        .await?;
        match result {
            ResultOfWaitForTransaction::Complete(transaction) => {
                emit_event(&context, &params.callback, ||MessageProcessingEvent::TransactionReceived(transaction))
            return Ok(ResultOfProcessMessage { transaction }),
            }

            Result
            Ok(_) => Ok(ResultOfProcessMessage { transaction: None }),
            Err(err) => return Err(err?),
        }
        retry_count += 1;
    }
}
