/*
 * Copyright 2018-2020 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 *
 */

use super::blocks_walking::find_last_shard_block;
use crate::abi::Abi;
use crate::client::ClientContext;
use crate::encoding::{base64_decode, hex_decode};
use crate::error::ApiResult;
use crate::processing::internal::{get_message_expiration_time, get_message_id};
use crate::processing::types::{CallbackParams, ProcessingEvent};
use crate::processing::Error;
use std::sync::Arc;
use ton_sdk::Contract;

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub struct ParamsOfSendMessage {
    /// Message BOC.
    pub message: String,

    /// Optional message ABI.
    ///
    /// If this parameter is specified and the message has the
    /// `expire` header then expiration time will be checked against
    /// the current time to prevent an unnecessary sending.
    ///
    /// The `message already expired` error will be returned in this
    /// case.
    ///
    /// Note that specifying `abi` for ABI compliant contracts is
    /// strongly recommended due to choosing proper processing
    /// strategy.
    pub abi: Option<Abi>,

    /// Processing callback.
    pub events_handler: Option<CallbackParams>,
}

#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug)]
pub struct ResultOfSendMessage {
    /// Shard block related to the message dst account before the
    /// message had been sent.
    ///
    /// This block id must be used as a parameter of the
    /// `wait_for_transaction`.
    pub shard_block_id: String,
}

#[function_info]
pub async fn send_message(
    context: Arc<ClientContext>,
    params: ParamsOfSendMessage,
) -> ApiResult<ResultOfSendMessage> {
    // Check message
    let message_boc = base64_decode(&params.message)?;
    let message = Contract::deserialize_message(&message_boc)
        .map_err(|err| Error::invalid_message_boc(err))?;
    let message_id = get_message_id(&message)?;
    let hex_message_id = hex::encode(&message_id);

    let address = message
        .dst()
        .ok_or(Error::message_has_not_destination_address())?;

    let message_expiration_time =
        get_message_expiration_time(context.clone(), params.abi.as_ref(), &params.message)?;

    if let Some(message_expiration_time) = message_expiration_time {
        if message_expiration_time <= context.env.now_ms() {
            return Err(Error::message_already_expired());
        }
    }

    // Fetch current shard block
    if let Some(cb) = &params.events_handler {
        ProcessingEvent::WillFetchFirstBlock {}.emit(&context, cb)
    }
    let shard_block_id = match find_last_shard_block(&context, &address).await {
        Ok(block) => block.to_string(),
        Err(err) => {
            let error = Error::fetch_first_block_failed(err, &hex_message_id);
            if let Some(cb) = &params.events_handler {
                ProcessingEvent::FetchFirstBlockFailed {
                    error: error.clone(),
                }
                .emit(&context, cb)
            }
            return Err(error);
        }
    };

    // Send
    if let Some(cb) = &params.events_handler {
        ProcessingEvent::WillSend {
            shard_block_id: shard_block_id.clone(),
            message_id: hex_message_id.clone(),
            message: params.message.clone(),
        }
        .emit(&context, cb)
    }
    let send_result = context
        .get_client()?
        .send_message(&hex_decode(&message_id)?, &message_boc)
        .await;
    if let Some(cb) = &params.events_handler {
        match send_result {
            Ok(_) => ProcessingEvent::DidSend {
                shard_block_id: shard_block_id.clone(),
                message_id: hex_message_id.clone(),
                message: params.message.clone(),
            },
            Err(error) => ProcessingEvent::SendFailed {
                shard_block_id: shard_block_id.clone(),
                message_id: hex_message_id.clone(),
                message: params.message.clone(),
                error: Error::send_message_failed(error, &hex_message_id, &shard_block_id),
            },
        }
        .emit(&context, cb)
    }

    Ok(ResultOfSendMessage { shard_block_id })
}
