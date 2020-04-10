/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::json_helper;
use crate::types::StringId;
use ton_types::Result;

#[cfg(feature = "node_interaction")]
use crate::node_client::NodeClient;
#[cfg(feature = "node_interaction")]
use crate::types::MESSAGES_TABLE_NAME;
#[cfg(feature = "node_interaction")]
use crate::error::SdkError;

use ton_types::{SliceData, Cell};
use ton_block::{
    CommonMsgInfo, Message as TvmMessage
};
use ton_block::GetRepresentationHash;


#[derive(Deserialize, Debug, PartialEq, Clone)]
pub enum MessageType {
    Internal,
    ExternalInbound,
    ExternalOutbound,
    Unknown,
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Unknown
    }
}

pub type MessageId = StringId;

#[derive(Debug, Deserialize, Default)]
pub struct Message {
    pub id: MessageId,
    #[serde(deserialize_with = "json_helper::deserialize_tree_of_cells_opt_cell")]
    pub body: Option<Cell>,
    #[serde(deserialize_with = "json_helper::deserialize_message_type")]
    pub msg_type: MessageType,
}

#[cfg(feature = "node_interaction")]
const MESSAGE_FIELDS: &str = r#"
    id
    body
    msg_type
"#;

// The struct represents sent message and allows to access their properties.
#[allow(dead_code)]
impl Message {

    // Asynchronously loads a Message instance or None if message with given id is not exists
    #[cfg(feature = "node_interaction")]
    pub async fn load(client: &NodeClient, id: &MessageId) -> Result<Option<Message>> {
        let value = client.load_record_fields(
            MESSAGES_TABLE_NAME,
            &id.to_string(),
            MESSAGE_FIELDS).await?;

        if value == serde_json::Value::Null {
            Ok(None)
        } else {
            Ok(Some(serde_json::from_value(value)
                .map_err(|err| SdkError::InvalidData {
                    msg: format!("error parsing message: {}", err)
                })?))
        }
    }

    // Asynchronously loads a Message's json representation 
    // or null if message with given id is not exists
    #[cfg(feature = "node_interaction")]
    pub async fn load_json(client: &NodeClient, id: MessageId) -> Result<String> {
        client.load_record_fields(
            MESSAGES_TABLE_NAME,
            &id.to_string(),
            MESSAGE_FIELDS)
                .await
                .map(|val| val.to_string())
    }

    pub fn with_msg(tvm_msg: &TvmMessage) -> Result<Self> {
        let mut msg = Self::default();
        msg.id = tvm_msg.hash()?.as_slice()[..].into();
        msg.body = tvm_msg.body().map(|slice| slice.into_cell());

        msg.msg_type = match tvm_msg.header() {
            CommonMsgInfo::IntMsgInfo(_) => MessageType::Internal,
            CommonMsgInfo::ExtInMsgInfo(_) => MessageType::ExternalInbound,
            CommonMsgInfo::ExtOutMsgInfo(_) => MessageType::ExternalOutbound
        };

        Ok(msg)
    }

    // Returns message's identifier
    pub fn id(&self) -> MessageId {
        // On client side id is ready allways. It is never be calculated, just returned.
        self.id.clone()
    }

    // Returns message's body (as tree of cells) or None if message doesn't have once
    pub fn body(&self) -> Option<SliceData> {
        self.body.clone().map(|cell| cell.into())
    }

    // Returns message's type
    pub fn msg_type(&self) -> MessageType {
        self.msg_type.clone()
    }
}