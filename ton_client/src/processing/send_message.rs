/*
 * Copyright 2018-2021 TON Labs LTD.
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
use crate::boc::internal::{deserialize_object_from_boc, DeserializedObject};
use crate::client::ClientContext;
use crate::encoding::{base64_decode, hex_decode};
use crate::error::{AddNetworkUrl, ClientResult};
use crate::net::EndpointStat;
use crate::processing::internal::get_message_expiration_time;
use crate::processing::types::ProcessingEvent;
use crate::processing::Error;
use std::sync::Arc;
use ever_block::{Message, MsgAddressInt};

#[derive(Serialize, Deserialize, ApiType, Default, Debug, Clone)]
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

    /// Flag for requesting events sending.
    /// Default is `false`.
    #[serde(default)]
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

    /// The list of endpoints to which the message was sent.
    ///
    /// This list id must be used as a parameter of the
    /// `wait_for_transaction`.
    pub sending_endpoints: Vec<String>,
}

#[derive(Clone)]
struct SendingMessage {
    serialized: String,
    _deserialized: DeserializedObject<Message>,
    id: String,
    body: Vec<u8>,
    dst: MsgAddressInt,
}

impl SendingMessage {
    fn new(
        context: &Arc<ClientContext>,
        serialized: &str,
        abi: Option<&Abi>,
    ) -> ClientResult<Self> {
        // Check message
        let deserialized =
            deserialize_object_from_boc::<Message>(&context, serialized, "message")?;
        let id = deserialized.cell.repr_hash().as_hex_string();
        let dst = deserialized
            .object
            .dst()
            .ok_or(Error::message_has_not_destination_address())?;

        let message_expiration_time =
            get_message_expiration_time(context.clone(), abi, &serialized)?;
        if let Some(message_expiration_time) = message_expiration_time {
            if message_expiration_time <= context.env.now_ms() {
                return Err(Error::message_already_expired());
            }
        }
        let body = base64_decode(serialized)?;
        Ok(Self {
            serialized: serialized.to_string(),
            _deserialized: deserialized,
            id,
            body,
            dst,
        })
    }

    async fn prepare_to_send<F: futures::Future<Output = ()> + Send>(
        &self,
        context: &Arc<ClientContext>,
        callback: &Option<impl Fn(ProcessingEvent) -> F + Send + Sync>,
    ) -> ClientResult<String> {
        if let Some(callback) = callback {
            callback(ProcessingEvent::WillFetchFirstBlock {
                message_id: self.id.to_string(),
                message_dst: self.dst.to_string(),
            })
            .await;
        }
        let shard_block_id = match find_last_shard_block(&context, &self.dst, None).await {
            Ok(block) => block.to_string(),
            Err(err) => {
                if let Some(callback) = &callback {
                    callback(ProcessingEvent::FetchFirstBlockFailed {
                        message_id: self.id.to_string(),
                        message_dst: self.dst.to_string(),
                        error: err.clone(),
                    })
                    .await;
                }
                return Err(Error::fetch_first_block_failed(err, &self.id));
            }
        };
        if let Some(callback) = &callback {
            callback(ProcessingEvent::WillSend {
                shard_block_id: shard_block_id.clone(),
                message_id: self.id.to_string(),
                message_dst: self.dst.to_string(),
                message: self.serialized.clone(),
            })
            .await;
        }
        Ok(shard_block_id)
    }

    async fn send(&self, context: &Arc<ClientContext>) -> ClientResult<Vec<String>> {
        let net = context.get_server_link()?;
        let endpoint = net.state().get_query_endpoint().await?;
        if endpoint.remp_enabled() {
            let address = endpoint.query_url.clone();
            return net
                .send_message(&hex_decode(&self.id)?, &self.body, Some(&endpoint))
                .await
                .add_endpoint_from_context(&context, &endpoint)
                .await
                .map(|_| vec![address]);
        }

        let addresses = context.get_server_link()?.get_addresses_for_sending().await;
        let mut last_result = None::<ClientResult<String>>;
        let succeeded_limit = context.config.network.sending_endpoint_count as usize;
        let mut succeeded = Vec::new();
        'sending: for selected_addresses in addresses.chunks(succeeded_limit) {
            let mut futures = vec![];
            for address in selected_addresses {
                let context = context.clone();
                let message = self.clone();
                futures.push(Box::pin(async move {
                    let result = message.send_to_address(context.clone(), address).await;
                    if result.is_err() {
                        context
                            .get_server_link()?
                            .update_stat(&[address.to_owned()], EndpointStat::MessageUndelivered)
                            .await;
                    }
                    result
                }));
            }
            for result in futures::future::join_all(futures).await {
                if let Ok(address) = &result {
                    succeeded.push(address.clone());
                    if succeeded.len() >= succeeded_limit {
                        break 'sending;
                    }
                }
                last_result = Some(result);
            }
        }
        if succeeded.len() > 0 {
            return Ok(succeeded);
        }
        Err(if let Some(Err(err)) = last_result {
            err
        } else {
            Error::block_not_found("no endpoints".to_string())
        })
    }

    async fn send_to_address(
        &self,
        context: Arc<ClientContext>,
        address: &str,
    ) -> ClientResult<String> {
        let link = context.get_server_link()?;
        let endpoint = if let Some(endpoint) = link.state().get_resolved_endpoint(address).await {
            endpoint
        } else {
            link.state().resolve_endpoint(address).await?
        };

        // Send
        link.send_message(&hex_decode(&self.id)?, &self.body, Some(&endpoint))
            .await
            .add_endpoint_from_context(&context, &endpoint)
            .await
            .map(|_| address.to_string())
    }
}

pub async fn send_message<F: futures::Future<Output = ()> + Send>(
    context: Arc<ClientContext>,
    params: ParamsOfSendMessage,
    callback: impl Fn(ProcessingEvent) -> F + Send + Sync + Clone,
) -> ClientResult<ResultOfSendMessage> {
    let message = SendingMessage::new(&context, &params.message, params.abi.as_ref())?;

    let callback = if params.send_events {
        Some(callback)
    } else {
        None
    };

    let shard_block_id = message.prepare_to_send(&context, &callback).await?;
    let result = message.send(&context).await;
    if let Some(callback) = &callback {
        callback(match &result {
            Ok(_) => ProcessingEvent::DidSend {
                shard_block_id: shard_block_id.to_string(),
                message_id: message.id.to_string(),
                message_dst: message.dst.to_string(),
                message: message.serialized.clone(),
            },
            Err(err) => ProcessingEvent::SendFailed {
                shard_block_id: shard_block_id.to_string(),
                message_id: message.id.to_string(),
                message_dst: message.dst.to_string(),
                message: message.serialized.clone(),
                error: Error::send_message_failed(err, &message.id, &shard_block_id),
            },
        })
        .await;
    }
    result.map(|sending_endpoints| ResultOfSendMessage {
        shard_block_id,
        sending_endpoints,
    })
}
