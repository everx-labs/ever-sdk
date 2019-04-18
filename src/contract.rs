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
    pub fn load(id: AccountId) -> SdkResult<NodeResponce<Contract>> {
        unimplemented!()
    }

    pub fn deploy(image: ContractImage) -> SdkResult<ChangesStream<ContractCallState>> {
        unimplemented!()

        // Deploy is call, but special message is constructed.
        // The message contains StateInit struct with code, public key and lib
        // and body with parameters for contract special method - constructor.

    }

    pub fn call() -> SdkResult<ChangesStream<ContractCallState>> {
        unimplemented!()

        // pack params into bag of cells via ABI
        // message_id = message's hash

        // send message by Kafka
        // (synchroniously - when responce will returned message will be in DB)

        // subscribe on updates from DB and return updates stream

    }

    fn send_message() -> SdkResult<()> {
        unimplemented!()

        // send message by Kafka
    }

    fn subscribe_updates() -> SdkResult<ChangesStream<ContractCallState>> {
        unimplemented!()
    }
}
