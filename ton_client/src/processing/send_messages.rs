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

use crate::client::ClientContext;
use crate::error::{AddNetworkUrl, ClientResult};
use serde_json::Value;
use std::sync::Arc;
use ton_client_processing::{MessageMonitoringParams, MonitoredMessage};

#[derive(Serialize, Deserialize, ApiType, Default, Debug, Clone)]
pub struct MessageSendingParams {
    /// BOC of the message, that must be sent to the blockchain.
    boc: String,

    /// Expiration time of the message.
    /// Must be specified as a UNIX timestamp in seconds.
    wait_until: u32,

    /// User defined data associated with this message.
    /// Helps to identify this message when user received message processing results.
    user_data: Option<Value>,
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug, Clone)]
pub struct ParamsOfSendMessages {
    /// Messages that must be sent to the blockchain.
    messages: Vec<MessageSendingParams>,

    /// Optional message monitor queue that starts monitoring for the processing
    /// results for sent messages.
    monitor_queue: Option<String>,
}

#[derive(Serialize, Deserialize, ApiType, Default, PartialEq, Debug)]
pub struct ResultOfSendMessages {
    /// Messages that was sent to the blockchain for execution.
    pub messages: Vec<MessageMonitoringParams>,
}

#[api_function]
/// Sends specified messages to the blockchain.
pub async fn send_messages(
    context: Arc<ClientContext>,
    params: ParamsOfSendMessages,
) -> ClientResult<ResultOfSendMessages> {
    let server_link = context.get_server_link()?;
    let endpoint = server_link.state().get_query_endpoint().await?;
    let messages = params
        .messages
        .iter()
        .map(|x| {
            (
                MonitoredMessage::get_boc_hash(&x.boc).unwrap(),
                x.boc.clone(),
            )
        })
        .collect();
    server_link
        .send_messages(messages, Some(&endpoint))
        .await
        .add_endpoint_from_context(&context, &endpoint)
        .await?;
    let messages = params
        .messages
        .into_iter()
        .map(|x| MessageMonitoringParams {
            message: MonitoredMessage::Boc { boc: x.boc },
            wait_until: x.wait_until,
            user_data: x.user_data,
        })
        .collect::<Vec<_>>();
    if let Some(queue) = params.monitor_queue {
        context
            .message_monitor
            .monitor_messages(&queue, messages.clone()).await?;
    }
    Ok(ResultOfSendMessages { messages })
}
