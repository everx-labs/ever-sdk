use crate::abi::{Abi, ParamsOfEncodeMessage};
use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::processing::internal::can_retry_expired_message;
use crate::processing::{
    send_message, wait_for_transaction, Error, ErrorCode, ParamsOfSendMessage,
    ParamsOfWaitForTransaction, ProcessingEvent, ResultOfProcessMessage,
};
use std::sync::Arc;

#[derive(Serialize, Deserialize, ApiType, Debug, Clone)]
pub enum MessageSource {
    /// Already prepared message BOC and optional ABI
    Encoded { 
        message: String, 
        abi: Option<Abi> 
    },
    EncodingParams(ParamsOfEncodeMessage),
}

impl MessageSource {
    pub(crate) async fn encode(
        &self,
        context: &Arc<ClientContext>,
    ) -> ClientResult<(String, Option<Abi>)> {
        Ok(match self {
            MessageSource::EncodingParams(params) => {
                if params.signer.is_external() {
                    return Err(Error::external_signer_must_not_be_used());
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
    /// Flag that enables/disables intermediate events
    pub send_events: bool,
}

pub async fn process_message<F: futures::Future<Output = ()> + Send + Sync>(
    context: Arc<ClientContext>,
    params: ParamsOfProcessMessage,
    callback: impl Fn(ProcessingEvent) -> F + Send + Sync + 'static,
) -> ClientResult<ResultOfProcessMessage> {
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
                send_events: params.send_events,
            },
            &callback,
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
            &callback,
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
                try_index
            }
        };
        try_index = try_index.checked_add(1).unwrap_or(try_index);
    }
}
