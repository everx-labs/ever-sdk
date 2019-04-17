use crate::*;
use std;
use tvm::types::UInt256;
use tvm::stack::SliceData;
use futures::stream::Stream;
use futures::future::Future;

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
    pub fn load(id: UInt256) -> SdkResult<NodeResponce<Message>> {
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