use crate::*;
use std::io::Read;
use tvm::stack::SliceData;
use tvm::types::AccountId;

pub struct ContractCallState {
    message_id: MessageId,
    message_state: MessageState,

    // Exists with MessageState::Proposed and MessageState::Finalized
    transaction: Option<TransactionId>
}

pub struct ContractImage {
    code: SliceData,
    data: SliceData,
    lib: SliceData,
}

impl ContractImage {
    fn new(code: &Read, data: Option<&Read>, library: Option<&Read>) -> SdkResult<Self> {
        unimplemented!()
    }

    fn id(&self) -> AccountId {
        unimplemented!()
    }
}

pub struct Contract {

}

impl Contract {
    fn load(id: AccountId) -> SdkResult<NodeResponce<Contract>> {
        unimplemented!()
    }

    fn deploy(image: ContractImage) -> SdkResult<ChangesStream<ContractCallState>> {
        unimplemented!()
    }

    fn call() -> SdkResult<ChangesStream<ContractCallState>> {
        unimplemented!()
    }
}
