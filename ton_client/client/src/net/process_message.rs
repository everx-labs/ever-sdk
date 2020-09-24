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
    /// Determine that callback must stay registered after operation has been finished.
    /// By default the callback will automatically unregistered.
    pub stay_registered: Option<bool>,
}

#[derive(Serialize, Deserialize, TypeInfo, Debug, Clone)]
pub enum MessageProcessingEvent {
    /// Notifies the app that the client will fetch the current
    /// shard block from network.
    /// Fetched block will be used later in waiting phase.
    WillFetchFirstBlock {},
    /// Notifies the app that the client has failed to fetch current shard block.
    /// Message processing has finished.
    FetchFirstBlockFailed { error: ApiError },
    /// Notifies the app that client will send the message to the network.
    WillSend {
        message_id: String,
        state: TransactionWaitingState,
    },
    /// Notifies the app that the sending operation was
    /// failed with network error.
    /// Processing will be continued at waiting phase because
    /// the message possibly has been delivered to the node.
    SendFailed {
        state: TransactionWaitingState,
        error: ApiError,
    },
    /// Notifies the app that the client will fetch the next
    /// shard block from the network.
    /// Event can occurs more than one time due to block walking procedure.
    WillFetchNextBlock {
        state: TransactionWaitingState,
    },
    /// Notifies the app that the next block can't be fetched due to error.
    /// Processing will be continued after `network_resume_timeout`.
    FetchNextBlockFailed {
        state: TransactionWaitingState,
        error: ApiError,
    },
    /// Notifies the app that the message was expired.
    /// Event occurs for messages with `expiration` replay protection.
    /// Processing will be continued at encoding message phase after
    /// `expiration_retries_timeout`.
    MessageHasExpired {
        state: TransactionWaitingState,
        error: ApiError,
    },
    /// Notifies the app that the client has received the transaction.
    /// Processing has finished.
    DidReceiveTransaction { transaction: ReceivedTransaction },
}

pub struct TransactionWaitingOptions {
    /// Limit the retries count for failed network requests.
    /// Negative value means infinite.
    /// Default is -1.
    pub network_retries_limit: Option<isize>,
    /// Timeout between retries of failed network operations.
    /// Default is 40000.
    pub network_retries_timeout: Option<u64>,
    /// Limit the retries count for expired messages.
    /// Negative value means infinite.
    /// Default is 8.
    pub expiration_retries_limit: Option<isize>,
    /// Limit the retries count for expired messages.
    /// Default is 40000.
    pub expiration_retries_timeout: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TransactionWaitingState {
    /// The last shard block received before the message was sent
    /// or the last shard block checked for the resulting transaction
    /// after the message was sent.
    pub last_checked_block_id: String,
    /// The time when the message was sent.
    pub message_sending_time: u32,
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
    pub waiting_state: TransactionWaitingState,
}

pub async fn send_message(
    context: Arc<ClientContext>,
    params: ParamsOfSendMessage,
) -> ApiResult<ResultOfSendMessage> {
    // Check for already expired
    let now = context.now();
    if let Some(message_expiration_time) = params.message_expiration_time {
        if message_expiration_time <= now {
            return Err(Error::message_already_expired());
        }
    }

    // Encode message
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

    // Send
    let mut transaction_waiting_state = TransactionWaitingState {
        expiration,
        message_sending_time: context.now(),
    };
    emit_event(&context, &params.callback, || {
        MessageProcessingEvent::SendMessage {
            transaction_waiting_state: transaction_waiting_state.clone(),
        }
    });
    if let Err(error) = client.send_message(&id, &boc).await? {
        emit_event(&context, &params.callback, || {
            MessageProcessingEvent::SendMessageFailed {
                transaction_waiting_state: transaction_waiting_state.clone(),
                error,
            }
        })
    }
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
    Incomplete(TransactionWaitingState),
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

    let message = Contract::deserialize_message(&base64_decode(&params.message)?)?;
    let id = get_message_id(&message)?;
    let address = message
        .dst()
        .ok_or(Error::message_has_not_destination_address())?;
    let mut transaction = Value::Null;
    let add_timeout = client.config().message_processing_timeout();
    let mut waiting_state = params.transaction_waiting_state;
    loop {
        let now = context.now();
        let timeout = std::cmp::max(stop_time, now) - now + add_timeout;
        emit_event(&context, &params.callback, || {
            MessageProcessingEvent::FetchBlock {}
        });
        let result = Block::wait_next_block(
            client,
            &waiting_state.last_block_id,
            &address,
            Some(timeout),
        )
        .await;
        let block = match result {
            Err(err) => {
                emit_event(&context, &params.callback, || {
                    MessageProcessingEvent::FetchBlockFailed { error }
                });
                if let Some(&SdkError::WaitForTimeout) = err.downcast_ref::<SdkError>() {
                    if infinite_wait {
                        continue;
                    }
                    return Err(Error::fetch_block_failed(
                        &message_id,
                        &waiting_state,
                        timeout,
                    ));
                } else if let Some(GraphiteError::NetworkError(_)) =
                    err.downcast_ref::<GraphiteError>()
                {
                    if infinite_wait {
                        futures_timer::Delay::new(std::time::Duration::from_secs(1)).await;
                        continue;
                    }
                    return Err(Error::fetch_block_failed(
                        &message_id,
                        &waiting_state,
                        timeout,
                    ));
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
                emit_event(&context, &params.callback, || {
                    MessageProcessingEvent::TransactionReceived {
                        transaction: transaction.clone(),
                    }
                });
                return Ok(ResultOfProcessMessage { transaction });
            }

            ResultOfWaitForTransaction::Incomplete(waiting_state) => {}
        }
        retry_count += 1;
    }
}
