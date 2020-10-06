use crate::abi::{Abi, ParamsOfEncodeMessage};
use crate::client::ClientContext;
use crate::dispatch::Callback;
use crate::error::ApiResult;
use crate::processing::internal::can_retry_expired_message;
use crate::processing::types::{ProcessingEvent, ProcessingResponseType, TransactionOutput};
use crate::processing::{
    send_message, wait_for_transaction, ErrorCode, ParamsOfSendMessage, ParamsOfWaitForTransaction,
};
use std::sync::Arc;

#[derive(Serialize, Deserialize, ApiType, Debug)]
pub enum MessageSource {
    Encoded { message: String, abi: Option<Abi> },
    AbiEncodingParams(ParamsOfEncodeMessage),
}

#[derive(Serialize, Deserialize, ApiType, Debug)]
pub struct ParamsOfProcessMessage {
    /// Message source.
    pub message: MessageSource,

    /// Flag for requesting events sending
    pub send_events: bool
}

/// Sends message to the network and monitors network for a result of
/// message processing.
#[api_function]
pub(crate) async fn process_message(
    context: Arc<ClientContext>,
    params: ParamsOfProcessMessage,
    callback: std::sync::Arc<Callback>,
) -> ApiResult<TransactionOutput> {
    let callback = move |result: ProcessingEvent| {
        callback.call(result, ProcessingResponseType::ProcessingEvent as u32);
        futures::future::ready(())
    };

    process_message(context, params, callback).await
}

/// Sends message to the network and monitors network for a result of
/// message processing.
pub async fn process_message_rust<F: futures::Future<Output = ()> + Send + Sync>(
    context: Arc<ClientContext>,
    params: ParamsOfProcessMessage,
    callback: impl Fn(ProcessingEvent) -> F + Send + Sync + 'static,
) -> ApiResult<TransactionOutput> {
    let abi = match &params.message {
        MessageSource::Encoded { abi, .. } => abi.clone(),
        MessageSource::AbiEncodingParams(encode_params) => Some(encode_params.abi.clone()),
    };
    let is_message_encodable = if let MessageSource::AbiEncodingParams(_) = params.message {
        true
    } else {
        false
    };

    let mut try_index = 0;
    loop {
        // Encode (or use encoded) message
        let message = match &params.message {
            MessageSource::Encoded { message, .. } => message.clone(),
            MessageSource::AbiEncodingParams(encode_params) => {
                let mut encode_params = encode_params.clone();
                encode_params.processing_try_index = Some(try_index);
                crate::abi::encode_message(context.clone(), encode_params)
                    .await?
                    .message
            }
        };

        // Send
        let shard_block_id = send_message(
            context.clone(),
            ParamsOfSendMessage {
                message: message.clone(),
                abi: abi.clone(),
                send_events: params.send_events,
            },
            &callback
        )
        .await?
        .shard_block_id;

        let wait_for = wait_for_transaction(
            context.clone(),
            ParamsOfWaitForTransaction {
                message: message.clone(),
                send_events: params.send_events,
                abi: abi.clone(),
                shard_block_id: shard_block_id.clone(),
            },
            &callback
        )
        .await;

        match wait_for {
            Ok(output) => {
                // Waiting is complete, return output
                return Ok(output);
            }
            Err(err) => {
                let can_retry = err.code == ErrorCode::MessageExpired as isize
                    && is_message_encodable
                    && can_retry_expired_message(&context, &mut try_index);
                if !can_retry {
                    // Waiting error is unrecoverable, return it
                    return Err(err);
                }
                // Waiting is failed but we can retry
            }
        };
    }
}
