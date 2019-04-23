use tvm::types::UInt256;

pub type TransactionId = UInt256;

// TODO this enum should be imported from ton_node module
pub enum TransactionState {
    Proposed,
    Finalized,
    Refused,
}

pub struct Transaction {

}
/*
impl Transaction {
    fn load(id: TransactionId) -> SdkResult<NodeResponce<Transaction>> {
        unimplemented!()
    }

    fn state(&self) -> TransactionState {
        unimplemented!()
    }

    pub fn state_changes(&self) -> SdkResult<ChangesStream<TransactionState>> {
        unimplemented!()
    }

    fn in_message_id(&self) -> UInt256 {
        unimplemented!()
    }

    fn out_messages_id(&self) -> &Iterator<Item = UInt256> {
        unimplemented!()
    }
}
*/