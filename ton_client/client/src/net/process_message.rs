use crate::abi::ParamsOfEncodeMessage;
use crate::client::ClientContext;
use crate::error::{ApiError, ApiResult};
use serde_json::Value;
use std::sync::Arc;
use ton_sdk::{Contract, SdkMessage};

//-------------------------------------------------------------------------------- process_message

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub enum MessageSource {
    Message(String),
    EncodingParams(ParamsOfEncodeMessage),
}

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub struct CallbackParams {
    /// Callback ID.
    pub id: u32,
    /// Automatically unregister callback after process have been finished.
    pub unregister: bool,
}

#[derive(Serialize, Deserialize, TypeInfo, Debug, Clone)]
pub struct MessageProcessingContext {
    /// Last visited block during transaction waiting phase.
    pub last_block_id: String,
    /// Message send time.
    pub send_time: u32,
}

#[derive(Serialize, Deserialize, TypeInfo, Debug, Clone)]
pub enum MessageProcessingEvent {
    /// Reports that message BOC will be encoded.
    /// Event occurs only for `EncodingParams` messages.
    /// Event can occurs more than one time for messages with `expiration` replay protection.
    EncodeMessage,
    /// Reports that account related block will be fetched from network.
    /// Event occurs only for message with `expiration` replay protection.
    /// Fetched block will be used later in following transaction monitoring.
    FetchBlock,
    /// Reports that the starting block can't be fetched due to error.
    /// Messaged processing has finished.
    FetchBlockFailed(ApiError),
    /// Reports that message will be sent to the network.
    SendMessage(MessageProcessingContext),
    /// Reports that message sending failed due to network error.
    /// Processing will be continued at waiting phase.
    SendMessageFailed(ApiError, MessageProcessingContext),
    /// Reports that next account related block will be fetched from network.
    /// Event occurs only for message with `expiration` replay protection.
    /// Event can occurs more than one time due to block walking procedure.
    WaitFetchBlock(MessageProcessingContext),
    /// Reports that the next block can't be fetched due to error.
    /// Event occurs only for message with `expiration` replay protection.
    /// Processing will be continued after network resuming timeout.
    WaitFetchBlockFailed(ApiError, MessageProcessingContext),
    /// Reports that the message was expired.
    /// Event occurs for message with `expiration` replay protection.
    /// Processing will be continued after expiration retries timeout
    /// at phase of encoding message.
    MessageExpired(ApiError),
    /// Reports that the processing starts listening for a transaction.
    /// Event occurs for message without `expiration` replay protection.
    /// Processing will be continued after network resuming timeout.
    WaitTransaction,
    /// Reports that the transaction listening failed due to timeout or network error.
    /// Event occurs for message without `expiration` replay protection.
    /// Processing has finished with error.
    WaitTransactionFailed(ApiError),
    /// Reports that the transaction received.
    /// Event occurs for message without `expiration` replay protection.
    /// Processing has finished with error.
    TransactionCompleted(Value),
}

pub struct MessageMonitoringOptions {
    /// `true` if message processing must monitor network until the transaction appears.
    /// `false` if message processing must just send message to network.
    pub transaction_required: bool,
    /// Limit the retries count for expired messages.
    pub expiration_retries_limit: Option<u32>,
}

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub struct ParamsOfProcessMessage {
    /// Message source.
    pub message: MessageSource,
    /// Resuming context.
    pub context: Option<MessageProcessingContext>,
    /// Processing callback.
    pub callback: Option<CallbackParams>,
}

#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug)]
pub struct ResultOfProcessMessage {
    pub transaction: Option<Value>,
}

fn send_event(
    context: &Arc<ClientContext>,
    callback: &Option<CallbackParams>,
    event: fn() -> MessageProcessingEvent,
) {
    if let Some(callback) = callback {
        let _ = context.send_callback_result(callback.id.clone(), event());
    }
}

fn create_message(
    context: &Arc<ClientContext>,
    source: &MessageSource,
    callback: &Option<CallbackParams>,
) -> ApiError<SdkMessage> {
    let boc = match source {
        MessageSource::Message(boc) => boc.clone(),
        MessageSource::EncodingParams(encode_params) => {
            send_event(context, callback, || MessageProcessingEvent::EncodeMessage);
            crate::abi::encode_message(context.clone(), encode_params.clone())?
        }
    };
}

pub async fn process_message(
    context: Arc<ClientContext>,
    params: ParamsOfProcessMessage,
) -> ApiResult<ResultOfProcessMessage> {
    let mut retry_count = 0;
    loop {
        let message = create_message(&context, &params.message, &params.callback)?;
        let result = Contract::process_message(client, &msg, true).await;
        match result {
            Ok(_) => Ok(ResultOfProcessMessage { transaction: None }),
            Err(err) => return Err(err?),
        }
        retry_count += 1;
    }
}
