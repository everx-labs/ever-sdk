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

// TODO need to realise to_string (or something else) for UInt256 in node
pub fn id_to_string(id: &UInt256) -> String {
    hex::encode(id.as_slice())
}

const MSG_TABLE_NAME: &str = "messages";

pub struct Message {

}

#[allow(dead_code)]
impl Message {
    fn load(_id: MessageId) -> SdkResult<Box<Stream<Item = Message, Error = SdkError>>> {
        unimplemented!()
    }

    pub fn load_json(id: MessageId) -> SdkResult<Box<Stream<Item = String, Error = SdkError>>> {

        let map = db_helper::load_record(MSG_TABLE_NAME, &id_to_string(&id))?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    fn state(&self) -> MessageState {
        unimplemented!()
    }

    fn id(&self) -> UInt256 {
        unimplemented!()
    }

    fn body(&self) -> Arc<CellData> {
        unimplemented!()
    }
}
