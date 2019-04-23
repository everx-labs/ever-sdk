use crate::*;
use tvm::types::UInt256;

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

pub struct Message {

}
/*
impl Message {
    pub fn load(id: MessageId) -> SdkResult<NodeResponce<Message>> {
        unimplemented!()
    }

    pub fn id(&self) -> MessageId {
        unimplemented!()
    }

    pub fn state(&self) -> MessageState {
        unimplemented!()
    }

    pub fn state_changes(&self) -> SdkResult<ChangesStream<MessageState>> {
        unimplemented!()
    }

    pub fn body(&self) -> SliceData {
        unimplemented!()
    }
}*/