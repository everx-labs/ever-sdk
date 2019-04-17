use crate::*;
use std;
use tvm::stack::SliceData;
use tvm::types::AccountId;

pub struct ContractCallResp {
    message_state: MessageState,

    // Exists with MessageState::Proposed and MessageState::Finalized
    transaction: Option<TransactionId>
}

pub struct ContractImage {
    code: SliceData,
    data: SliceData,
    //lib: SliceData,
}

impl ContractImage {
    fn from_file(filename: &str) -> SdkResult<Self> {
        unimplemented!()
    }

    fn from_code(code: &str) -> SdkResult<Self> {
        unimplemented!()
    }

    fn set_data(&mut self, data: &[u8]) -> SdkResult<()> {
        unimplemented!()
    }

    fn set_code(&mut self, code: &[u8]) -> SdkResult<()> {
        unimplemented!()
    }

    fn data(&self) -> SdkResult<&[u8]> {
        unimplemented!()
    }

    fn code(&self) -> SdkResult<&[u8]> {
        unimplemented!()
    }

    fn data_slice(&self) -> SdkResult<SliceData> {
        unimplemented!()
    }

    fn code_slice(&self) -> SdkResult<SliceData> {
        unimplemented!()
    }

    // library funcs....
}

pub struct Contract {

}

impl Contract {
    fn load(id: AccountId) -> SdkResult<NodeResponce<Transaction>> {
        unimplemented!()
    }

    fn deploy(image: ContractImage) -> SdkResult<ContractCallResp> {
        unimplemented!()
    }

    fn call() -> SdkResult<ContractCallResp> {
        unimplemented!()
    }
}
