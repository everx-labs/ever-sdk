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
*/

use crate::json_helper;
use crate::types::{StringId, grams_to_u64};
use ton_types::Result;

use ton_block::GetRepresentationHash;
use ton_block::{CommonMsgInfo, Message as TvmMessage};
use ton_types::{Cell, SliceData};

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
    #[serde(default, with = "json_helper::opt_cell")]
    pub body: Option<Cell>,
    #[serde(deserialize_with = "json_helper::deserialize_message_type")]
    pub msg_type: MessageType,
    #[serde(with = "json_helper::uint")]
    pub value: u64,
}

// The struct represents sent message and allows to access their properties.
#[allow(dead_code)]
impl Message {
    pub fn with_msg(tvm_msg: &TvmMessage) -> Result<Self> {
        let id = tvm_msg.hash()?.as_slice()[..].into();
        let body = tvm_msg.body().map(|slice| slice.into_cell());
        let value = tvm_msg
            .get_value()
            .map(|cc| grams_to_u64(&cc.grams))
            .transpose()?
            .unwrap_or(0);

        let msg_type = match tvm_msg.header() {
            CommonMsgInfo::IntMsgInfo(_) => MessageType::Internal,
            CommonMsgInfo::ExtInMsgInfo(_) => MessageType::ExternalInbound,
            CommonMsgInfo::ExtOutMsgInfo(_) => MessageType::ExternalOutbound,
        };

        Ok(Self {
            id,
            body,
            msg_type,
            value
        })
    }

    // Returns message's identifier
    pub fn id(&self) -> MessageId {
        // On client side id is ready allways. It is never be calculated, just returned.
        self.id.clone()
    }

    // Returns message's body (as tree of cells) or None if message doesn't have once
    pub fn body(&self) -> Option<SliceData> {
        self.body.clone().and_then(|cell| SliceData::load_cell(cell).ok())
    }

    // Returns message's type
    pub fn msg_type(&self) -> MessageType {
        self.msg_type.clone()
    }
}
