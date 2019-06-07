use crate::*;
use futures::stream::Stream;
use std::sync::Arc;
use tvm::stack::CellData;
use ton_block::{MessageId, MessageProcessingStatus};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum MessageType {
    Unknown,
    Internal,
    ExternalInbound,
    ExternalOutbound
}

pub const MSG_TABLE_NAME: &str = "messages";
pub const MSG_STATE_FIELD_NAME: &str = "status";

#[derive(Debug)]
pub struct Message {
    msg: ton_block::Message,
}

// The struct represents sent message and allows to access their properties.
#[allow(dead_code)]
impl Message {

    // Asynchronously loads a Message instance or None if message with given id is not exists
    pub fn load(id: MessageId) -> SdkResult<Box<Stream<Item = Option<Message>, Error = SdkError>>> {
        let map = db_helper::load_record(MSG_TABLE_NAME, &id.to_hex_string())?
            .and_then(|val| {
                if val == serde_json::Value::Null {
                    Ok(None)
                } else {
                    let msg: ton_block::Message = serde_json::from_value(val)
                        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing message: {}", err)))?;

                    Ok(Some(Message { msg }))
                }
            });

        Ok(Box::new(map))
    }

    // Asynchronously loads a Message's json representation 
    // or null if message with given id is not exists
    pub fn load_json(id: MessageId) -> SdkResult<Box<Stream<Item = String, Error = SdkError>>> {

        let map = db_helper::load_record(MSG_TABLE_NAME, &id.to_hex_string())?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    // Returns message's processing status
    pub fn status(&self) -> MessageProcessingStatus {
        self.msg.status.clone()
    }

    // Returns message's identifier
    pub fn id(&self) -> MessageId {
        self.msg.id.clone()
    }

    // Returns message's body (as tree of cells) or None if message doesn't have once
    pub fn body(&self) -> Option<Arc<CellData>> {
        self.msg.body.clone()
    }

    // Returns blockchain's message struct
    // Some node-specifed methods won't work. All TonStructVariant fields has Client variant.
    pub fn msg(&self) -> &ton_block::Message {
         &self.msg
    }

    // Returns message's type
    pub fn msg_type(&self) -> MessageType {
        match self.msg.header {
            ton_block::CommonMsgInfo::IntMsgInfo(_) => MessageType::Internal,
            ton_block::CommonMsgInfo::ExtInMsgInfo(_) => MessageType::ExternalInbound,
            ton_block::CommonMsgInfo::ExtOutMsgInfo(_) => MessageType::ExternalOutbound,
        }
    }
}