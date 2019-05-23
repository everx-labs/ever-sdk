use crate::*;
use tvm::types::UInt256;
use futures::stream::Stream;
use std::sync::Arc;
use tvm::stack::CellData;
use ton_block::{MessageId, MessageProcessingStatus};

// TODO need to realise to_string (or something else) for UInt256 in node
pub fn id_to_string(id: &UInt256) -> String {
    hex::encode(id.as_slice())
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum MessageType {
    Unknown,
    Internal,
    ExternalInbound,
    ExternalOutbound
}

const MSG_TABLE_NAME: &str = "messages";

#[derive(Debug)]
pub struct Message {
    msg: ton_block::Message,
}

#[allow(dead_code)]
impl Message {

    pub fn load(id: MessageId) -> SdkResult<Box<Stream<Item = Message, Error = SdkError>>> {
        let map = db_helper::load_record(MSG_TABLE_NAME, &id_to_string(&id))?
            .and_then(|val| {
                let msg: ton_block::Message = serde_json::from_value(val)
                    .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing message: {}", err)))?;

                Ok(Message { msg })
            });

        Ok(Box::new(map))
    }

    pub fn load_json(id: MessageId) -> SdkResult<Box<Stream<Item = String, Error = SdkError>>> {

        let map = db_helper::load_record(MSG_TABLE_NAME, &id_to_string(&id))?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    pub fn status(&self) -> MessageProcessingStatus {
        self.msg.status.clone()
    }

    pub fn id(&self) -> MessageId {
        self.msg.id.clone()
    }

    pub fn body(&self) -> Option<Arc<CellData>> {
        self.msg.body.clone()
    }

    pub fn msg_type(&self) -> MessageType {
        match self.msg.header {
            ton_block::CommonMsgInfo::IntMsgInfo(_) => MessageType::Internal,
            ton_block::CommonMsgInfo::ExtInMsgInfo(_) => MessageType::ExternalInbound,
            ton_block::CommonMsgInfo::ExtOutMsgInfo(_) => MessageType::ExternalOutbound,
        }
    }
}