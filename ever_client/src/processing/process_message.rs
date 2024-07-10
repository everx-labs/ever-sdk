use crate::abi::ParamsOfEncodeMessage;
use crate::client::ClientContext;
use crate::error::{AddNetworkUrl, ClientResult};
use crate::processing::internal::can_retry_expired_message;
use crate::processing::{
    send_message, wait_for_transaction, ErrorCode, ParamsOfSendMessage, ParamsOfWaitForTransaction,
    ProcessingEvent, ResultOfProcessMessage, ResultOfSendMessage,
};
use crate::tvm::StdContractError;
use std::sync::Arc;

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ParamsOfProcessMessage {
    /// Message encode parameters.
    pub message_encode_params: ParamsOfEncodeMessage,

    /// Flag for requesting events sending.
    /// Default is `false`.
    #[serde(default)]
    pub send_events: bool,
}

pub async fn process_message<F: futures::Future<Output = ()> + Send>(
    context: Arc<ClientContext>,
    params: ParamsOfProcessMessage,
    callback: impl Fn(ProcessingEvent) -> F + Send + Sync + 'static,
) -> ClientResult<ResultOfProcessMessage> {
    let abi = params.message_encode_params.abi.clone();

    let mut try_index = 0;
    loop {
        // Encode message
        let mut encode_params = params.message_encode_params.clone();
        encode_params.processing_try_index = Some(try_index);
        let message = crate::abi::encode_message(context.clone(), encode_params)
            .await?;

        // Send
        let ResultOfSendMessage {
            shard_block_id,
            sending_endpoints,
        } = send_message(
            context.clone(),
            ParamsOfSendMessage {
                message: message.message.clone(),
                abi: Some(abi.clone()),
                send_events: params.send_events,
            },
            &callback,
        )
        .await
        .add_network_url_from_context(&context)
        .await?;

        let wait_for = wait_for_transaction(
            context.clone(),
            ParamsOfWaitForTransaction {
                message: message.message.clone(),
                send_events: params.send_events,
                abi: Some(abi.clone()),
                shard_block_id: shard_block_id.clone(),
                sending_endpoints: Some(sending_endpoints),
            },
            &callback,
        )
        .await
        .add_network_url_from_context(&context)
        .await;

        match wait_for {
            Ok(output) => {
                // Waiting is complete, return output
                return Ok(output);
            }
            Err(err) => {
                let local_exit_code = &err.data["local_error"]["data"]["exit_code"];
                let can_retry = err.code == ErrorCode::MessageExpired as u32
                    && (err.data["local_error"].is_null()
                        || local_exit_code == StdContractError::ReplayProtection as i32
                        || local_exit_code == StdContractError::ExtMessageExpired as i32)
                    && can_retry_expired_message(&context, try_index);
                if !can_retry {
                    // Waiting error is unrecoverable, return it
                    return Err(err);
                }
                if params.send_events {
                    callback(ProcessingEvent::MessageExpired {
                        message_id: message.message_id,
                        message_dst: message.address,
                        message: message.message,
                        error: err
                    }).await;
                }
                // Waiting is failed but we can retry
            }
        };
        try_index = try_index.checked_add(1).unwrap_or(try_index);
    }
}
