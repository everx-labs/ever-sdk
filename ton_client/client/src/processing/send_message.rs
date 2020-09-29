use crate::client::ClientContext;
use crate::encoding::{base64_decode, hex_decode};
use crate::error::{ApiResult};
use crate::processing::internal::get_message_id;
use crate::processing::types::{CallbackParams, ProcessingEvent, ProcessingState};
use crate::processing::Error;
use std::sync::Arc;
use ton_sdk::{
    Block, Contract,
};

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub struct ParamsOfSendMessage {
    /// Message BOC.
    pub message: String,
    /// Message expiration time.
    /// Used only for messages with `expiration` replay protection.
    pub message_expiration_time: Option<u64>,
    /// Processing callback.
    pub callback: Option<CallbackParams>,
}

#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug)]
pub struct ResultOfSendMessage {
    pub processing_state: ProcessingState,
}

#[method_info(name = "processing.send_message")]
pub async fn send_message(
    context: Arc<ClientContext>,
    params: ParamsOfSendMessage,
) -> ApiResult<ResultOfSendMessage> {
    // Check for already expired
    {
        if let Some(message_expiration_time) = params.message_expiration_time {
            if message_expiration_time <= context.now_millis() {
                return Err(Error::message_already_expired());
            }
        }
    }

    // Check message
    let message_boc = base64_decode(&params.message)?;
    let message = Contract::deserialize_message(&message_boc)
        .map_err(|err| Error::invalid_message_boc(err))?;
    let message_id = get_message_id(&message)?;
    let hex_message_id = hex::encode(&message_id);

    let address = message
        .dst()
        .ok_or(Error::message_has_not_destination_address())?;

    // Fetch current shard block
    let client = context.get_client()?;
    if let Some(cb) = &params.callback {
        ProcessingEvent::WillFetchFirstBlock {}.emit(&context, cb)
    }
    let last_checked_block_id = match Block::find_last_shard_block(client, &address).await {
        Ok(block) => block.to_string(),
        Err(err) => {
            let error = Error::fetch_first_block_failed(err, &hex_message_id);
            if let Some(cb) = &params.callback {
                ProcessingEvent::FetchFirstBlockFailed {
                    error: error.clone(),
                }
                .emit(&context, cb)
            }
            return Err(error);
        }
    };

    // Initialize processing state
    let processing_state = ProcessingState {
        last_checked_block_id,
        message_sending_time: context.now_millis(),
    };

    // Send
    if let Some(cb) = &params.callback {
        ProcessingEvent::WillSend {
            processing_state: processing_state.clone(),
            message_id: hex_message_id.clone(),
            message: params.message.clone(),
        }
        .emit(&context, cb)
    }
    let send_result = client
        .send_message(&hex_decode(&message_id)?, &message_boc)
        .await;
    if let Some(cb) = &params.callback {
        match send_result {
            Ok(_) => ProcessingEvent::DidSend {
                processing_state: processing_state.clone(),
                message_id: hex_message_id.clone(),
                message: params.message.clone(),
            },
            Err(error) => ProcessingEvent::SendFailed {
                processing_state: processing_state.clone(),
                message_id: hex_message_id.clone(),
                message: params.message.clone(),
                error: Error::send_message_failed(error, &hex_message_id, &processing_state),
            },
        }
        .emit(&context, cb)
    }

    Ok(ResultOfSendMessage { processing_state })
}
