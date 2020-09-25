use crate::abi::ParamsOfEncodeMessage;
use crate::client::ClientContext;
use crate::encoding::base64_decode;
use crate::error::{ApiError, ApiResult};
use crate::net::internal::{emit_event, get_message_id};
use crate::net::types::{CallbackParams, MessageProcessingEvent, TransactionWaitingState};
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
    let message = Contract::deserialize_message(&message_boc)?;
    let message_id = get_message_id(&message)?;
    let address = message
        .dst()
        .ok_or(Error::message_has_not_destination_address())?;

    let client = context.get_client()?;
    emit_event(&context, &params.callback, || {
        MessageProcessingEvent::WillFetchFirstBlock {}
    });
    let mut waiting_state = TransactionWaitingState {
        last_checked_block_id: Block::find_last_shard_block(client, &address).await?.into(),
        message_sending_time: context.now_ms(),
    };

    // Send
    emit_event(&context, &params.callback, || {
        MessageProcessingEvent::WillSend {
            waiting_state: waiting_state.clone(),
            message_id: hex::encode(&message_id),
        }
    });
    if let Err(error) = client.send_message(&message_id, &message_boc).await? {
        emit_event(&context, &params.callback, || {
            MessageProcessingEvent::SendFailed {
                waiting_state: waiting_state.clone(),
                error,
            }
        })
    }

    Ok(ResultOfSendMessage { waiting_state })
}
