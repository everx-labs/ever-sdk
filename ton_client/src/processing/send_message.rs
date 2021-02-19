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
use crate::boc::internal::deserialize_object_from_boc;
use crate::client::ClientContext;
use crate::encoding::hex_decode;
use crate::error::{AddNetworkUrl, ClientResult};
use crate::processing::internal::get_message_expiration_time;
use crate::processing::types::ProcessingEvent;
use crate::processing::Error;
use std::sync::Arc;

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ParamsOfSendMessage {
    /// Message BOC.
    pub message: String,

    /// Optional message ABI.
    ///
    /// If this parameter is specified and the message has the
    /// `expire` header then expiration time will be checked against
    /// the current time to prevent unnecessary sending of already expired message.
    ///
    /// The `message already expired` error will be returned in this
    /// case.
    ///
    /// Note, that specifying `abi` for ABI compliant contracts is
    /// strongly recommended, so that proper processing strategy can be
    /// chosen.
    pub abi: Option<Abi>,

    /// Flag for requesting events sending
    pub send_events: bool,
}

#[derive(Serialize, Deserialize, ApiType, Default, PartialEq, Debug)]
pub struct ResultOfSendMessage {
    /// The last generated shard block of the message destination account before the
    /// message was sent.
    ///
    /// This block id must be used as a parameter of the
    /// `wait_for_transaction`.
    pub shard_block_id: String,
}

pub async fn send_message<F: futures::Future<Output = ()> + Send>(
    context: Arc<ClientContext>,
    params: ParamsOfSendMessage,
    callback: impl Fn(ProcessingEvent) -> F + Send + Sync,
) -> ClientResult<ResultOfSendMessage> {
    // Check message
    let message = deserialize_object_from_boc::<ton_block::Message>(&context, &params.message, "message").await?;
    let message_id = message.cell.repr_hash().to_hex_string();

    let address = message
        .object
        .dst()
        .ok_or(Error::message_has_not_destination_address())?;

    let message_expiration_time =
        get_message_expiration_time(context.clone(), params.abi.as_ref(), &params.message).await?;

    if let Some(message_expiration_time) = message_expiration_time {
        if message_expiration_time <= context.env.now_ms() {
            return Err(Error::message_already_expired());
        }
    }

    // Fetch current shard block
    if params.send_events {
        callback(ProcessingEvent::WillFetchFirstBlock {}).await;
    }
    let shard_block_id = match find_last_shard_block(&context, &address).await {
        Ok(block) => block.to_string(),
        Err(err) => {
            let error = Error::fetch_first_block_failed(err, &message_id);
            if params.send_events {
                callback(ProcessingEvent::FetchFirstBlockFailed {
                    error: error.clone(),
                })
                .await;
            }
            return Err(error);
        }
    };

    // Send
    if params.send_events {
        callback(ProcessingEvent::WillSend {
            shard_block_id: shard_block_id.clone(),
            message_id: message_id.clone(),
            message: params.message.clone(),
        })
        .await;
    }
    let send_result = context
        .get_server_link()?
        .send_message(&hex_decode(&message_id)?, &message.boc.bytes("message")?)
        .await
        .add_network_url_from_context(&context)
        .await?;
    if params.send_events {
        let event = match send_result {
            None => ProcessingEvent::DidSend {
                shard_block_id: shard_block_id.clone(),
                message_id: message_id.clone(),
                message: params.message.clone(),
            },
            Some(error) => ProcessingEvent::SendFailed {
                shard_block_id: shard_block_id.clone(),
                message_id: message_id.clone(),
                message: params.message.clone(),
                error: Error::send_message_failed(error, &message_id, &shard_block_id),
            },
        };
        callback(event).await;
    }

    Ok(ResultOfSendMessage { shard_block_id })
}
