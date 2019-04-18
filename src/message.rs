use crate::*;
use std;
use tvm::types::UInt256;
use tvm::stack::SliceData;
use futures::stream::Stream;
use futures::future::Future;

pub type MessageId = UInt256;

// TODO this enum should be imported from ton_node module
pub enum MessageState {
    Queued,
    Processing,
    Proposed,
    Finalized,
    Refused,
}

pub struct Message {

}

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
}