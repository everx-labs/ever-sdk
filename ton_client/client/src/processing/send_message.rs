use crate::abi::ParamsOfEncodeMessage;
use crate::client::ClientContext;
use crate::encoding::base64_decode;
use crate::error::{ApiError, ApiResult};
use crate::processing::internal::{get_message_id};
use crate::processing::types::{CallbackParams, ProcessingEvent, ProcessingState};
use crate::processing::Error;
use serde_json::Value;
use std::sync::Arc;
use ton_block::MsgAddressInt;
use ton_sdk::node_client::MAX_TIMEOUT;
use ton_sdk::types::TRANSACTIONS_TABLE_NAME;
use ton_sdk::{
    Block, BlockId, Contract, MessageId, MessageProcessingState, NodeClient, ReceivedTransaction,
    SdkError, SdkMessage, Transaction,
};

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
    pub waiting_state: ProcessingState,
}

#[method_info(name = "processing.send_message")]
pub async fn send_message(
    context: Arc<ClientContext>,
    params: ParamsOfSendMessage,
) -> ApiResult<ResultOfSendMessage> {
    // Check for already expired
    {
        let now = context.now();
        if let Some(message_expiration_time) = params.message_expiration_time {
            if message_expiration_time <= now {
                return Err(Error::message_already_expired());
            }
        }
    }

    // Encode message
    let message_boc = base64_decode(&params.message)?;
    let message = Contract::deserialize_message(&message_boc)
        .map_err(|err| Error::invalid_message_boc(err))?;
    let message_id = get_message_id(&message)?;
    let address = message
        .dst()
        .ok_or(Error::message_has_not_destination_address())?;

    let client = context.get_client()?;
    if let Some(cb) = &params.callback {
        ProcessingEvent::WillFetchFirstBlock {}.emit(&context, cb)
    }
    let waiting_state = ProcessingState {
        last_checked_block_id: Block::find_last_shard_block(client, &address)
            .await
            .map_err(|err| Error::fetch_first_block_failed(err, &hex::encode(&message_id)))?
            .to_string(),
        message_sending_time: context.now_millis(),
    };

    // Send
    if let Some(cb) = &params.callback {
        ProcessingEvent::WillSend {
            waiting_state: waiting_state.clone(),
            message_id: hex::encode(&message_id),
        }
        .emit(&context, cb)
    }
    if let Err(error) = client.send_message(&message_id, &message_boc).await {
        if let Some(cb) = &params.callback {
            ProcessingEvent::SendFailed {
                waiting_state: waiting_state.clone(),
                error: Error::send_message_failed(error, &hex::encode(message_id), &waiting_state),
            }
            .emit(&context, cb)
        }
    }

    Ok(ResultOfSendMessage { waiting_state })
}
