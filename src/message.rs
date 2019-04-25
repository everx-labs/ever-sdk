use crate::*;
use tvm::types::UInt256;
use futures::stream::Stream;
use std::sync::Arc;
use tvm::stack::CellData;

pub type MessageId = UInt256;

// TODO this enum should be imported from ton_node module
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum MessageState {
    Unknown,
    Queued,
    Processing,
    Proposed,
    Finalized,
    Refused,
}


// TODO this enum should be imported from ton_node module
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum MessageType {
    Unknown,
    OutboundExternal,
    // ...
}

// TODO need to realise to_string (or something else) for UInt256 in node
pub fn id_to_string(id: &UInt256) -> String {
    hex::encode(id.as_slice())
}

const MSG_TABLE_NAME: &str = "messages";

pub struct Message {
    id: MessageId,

}

#[allow(dead_code)]
impl Message {

    pub fn load(id: MessageId) -> SdkResult<Box<Stream<Item = Message, Error = SdkError>>> {
        let map = db_helper::load_record(MSG_TABLE_NAME, &id_to_string(&id))?
            .map(move |_val| Message { id: id.clone() }); // TODO parse json

        Ok(Box::new(map))
    }

    pub fn load_json(id: MessageId) -> SdkResult<Box<Stream<Item = String, Error = SdkError>>> {

        let map = db_helper::load_record(MSG_TABLE_NAME, &id_to_string(&id))?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    pub fn msg_type(&self) -> MessageType {
        unimplemented!()
    }

    pub fn state(&self) -> MessageState {
        unimplemented!()
    }

    pub fn id(&self) -> MessageId {
        self.id.clone()
    }

    pub fn body(&self) -> Arc<CellData> {
        unimplemented!()
    }
}
