use crate::abi::{Abi, ParamsOfEncodeMessage};
use crate::client::ClientContext;
use crate::error::ApiResult;
use crate::processing::internal::can_retry_expired_message;
use crate::processing::types::{CallbackParams, ResultOfProcessMessage};
use crate::processing::{send_message, wait_for_transaction, ErrorCode, ParamsOfSendMessage, ParamsOfWaitForTransaction, Error};
use std::sync::Arc;

#[derive(Serialize, Deserialize, ApiType, Debug, Clone)]
pub enum MessageSource {
    Encoded { message: String, abi: Option<Abi> },
    EncodingParams(ParamsOfEncodeMessage),
}

impl MessageSource {
    pub(crate) async fn encode(
        &self,
        context: &Arc<ClientContext>,
    ) -> ApiResult<(String, Option<Abi>)> {
        Ok(match self {
            MessageSource::EncodingParams(params) => {
                if params.signer.is_external() {
                    return Err(Error::external_signer_must_not_be_used())
                }
                let abi = params.abi.clone();
                (
                    crate::abi::encode_message(context.clone(), params.clone())
                        .await?
                        .message,
                    Some(abi),
                )
            }
            MessageSource::Encoded { abi, message } => (message.clone(), abi.clone()),
        })
    }
}

#[derive(Serialize, Deserialize, ApiType, Debug)]
pub struct ParamsOfProcessMessage {
    /// Message source.
    pub message: MessageSource,
    /// Processing callback.
    pub events_handler: Option<CallbackParams>,
}

/// Sends message to the network and monitors network for a result of
/// message processing.
#[api_function]
pub async fn process_message(
    context: Arc<ClientContext>,
    params: ParamsOfProcessMessage,
) -> ApiResult<ResultOfProcessMessage> {
    let abi = match &params.message {
        MessageSource::Encoded { abi, .. } => abi.clone(),
        MessageSource::EncodingParams(encode_params) => Some(encode_params.abi.clone()),
    };
    let is_message_encodable = if let MessageSource::EncodingParams(_) = params.message {
        true
    } else {
        false
    };

    let mut try_index = 0;
    loop {
        // Encode (or use encoded) message
        let message = match &params.message {
            MessageSource::Encoded { message, .. } => message.clone(),
            MessageSource::EncodingParams(encode_params) => {
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
                events_handler: params.events_handler.clone(),
            },
        )
        .await?
        .shard_block_id;

        let wait_for = wait_for_transaction(
            context.clone(),
            ParamsOfWaitForTransaction {
                message: message.clone(),
                events_handler: params.events_handler.clone(),
                abi: abi.clone(),
                shard_block_id: shard_block_id.clone(),
            },
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
                    && can_retry_expired_message(&context, try_index);
                if !can_retry {
                    // Waiting error is unrecoverable, return it
                    return Err(err);
                }
                // Waiting is failed but we can retry
            }
        };
        try_index = try_index.checked_add(1).unwrap_or(try_index);
    }
}
