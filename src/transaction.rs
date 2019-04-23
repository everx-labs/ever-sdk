use crate::*;
use tvm::types::UInt256;
use futures::stream::Stream;

pub type TransactionId = UInt256;

// TODO this enum should be imported from ton_node module
pub enum TransactionState {
    Proposed,
    Finalized,
    Refused,
}

pub struct Transaction {

}

const TR_TABLE_NAME: &str = "transactions";

#[allow(dead_code)]
impl Transaction {
    fn load(_id: TransactionId) -> SdkResult<Box<Stream<Item = Transaction, Error = SdkError>>> {
        unimplemented!()
    }

    pub fn load_json(id: TransactionId) -> SdkResult<Box<Stream<Item = String, Error = SdkError>>> {

        let map = db_helper::load_record(TR_TABLE_NAME, &id_to_string(&id))?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    fn state(&self) -> TransactionState {
        unimplemented!()
    }

    fn in_message_id(&self) -> UInt256 {
        unimplemented!()
    }

    fn out_messages_id(&self) -> &Iterator<Item = UInt256> {
        unimplemented!()
    }
}
