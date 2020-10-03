use crate::abi::{Abi, ParamsOfEncodeMessage};
use crate::client::ClientContext;
use crate::error::ApiResult;
use crate::processing::internal::can_retry_expired_message;
use crate::processing::types::{CallbackParams, TransactionOutput};
use crate::processing::{
    send_message, wait_for_transaction, ErrorCode, ParamsOfSendMessage, ParamsOfWaitForTransaction,
    ResultOfWaitForTransaction,
};
use std::sync::Arc;

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub enum MessageSource {
    Encoded { message: String, abi: Option<Abi> },
    AbiEncodingParams(ParamsOfEncodeMessage),
}

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub struct ParamsOfProcessMessage {
    /// Message source.
    pub message: MessageSource,
    /// Processing callback.
    pub callback: Option<CallbackParams>,
}

/// Sends message to the network and monitors network for a result of
/// message processing.
#[function_info]
pub async fn process_message(
    context: Arc<ClientContext>,
    params: ParamsOfProcessMessage,
) -> ApiResult<TransactionOutput> {
    let abi = match &params.message {
        MessageSource::Encoded { abi, .. } => abi.clone(),
        MessageSource::AbiEncodingParams(encode_params) => Some(encode_params.abi.clone()),
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
        let mut processing_state = send_message(
            context.clone(),
            ParamsOfSendMessage {
                message: message.clone(),
                abi: abi.clone(),
                callback: params.callback.clone(),
            },
        )
        .await?
        .processing_state;

        // Monitor network
        loop {
            match wait_for_transaction(
                context.clone(),
                ParamsOfWaitForTransaction {
                    message: message.clone(),
                    callback: params.callback.clone(),
                    abi: abi.clone(),
                    processing_state: processing_state.clone(),
                },
            )
            .await
            {
                Ok(result) => match result {
                    ResultOfWaitForTransaction::Complete(output) => {
                        return Ok(output);
                    }
                    ResultOfWaitForTransaction::Incomplete {
                        processing_state: incomplete_state,
                        ..
                    } => {
                        processing_state = incomplete_state;
                    }
                },
                Err(err) => {
                    if err.code == ErrorCode::MessageExpired as isize {
                        if can_retry_expired_message(&context, &mut try_index) {}
                    } else if err.code == ErrorCode::TransactionWaitTimeout as isize {
                        return Err(err);
                    }
                }
            }
        }
    }
}
