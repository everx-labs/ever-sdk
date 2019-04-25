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
    id: TransactionId,

}

const TR_TABLE_NAME: &str = "transactions";

#[allow(dead_code)]
impl Transaction {
    pub fn load(id: TransactionId) -> SdkResult<Box<Stream<Item = Transaction, Error = SdkError>>> {
        let map = db_helper::load_record(TR_TABLE_NAME, &id_to_string(&id))?
            .map(move |_val| Transaction { id: id.clone() }); // TODO parse json

        Ok(Box::new(map))
    }

    pub fn load_json(id: TransactionId) -> SdkResult<Box<Stream<Item = String, Error = SdkError>>> {

        let map = db_helper::load_record(TR_TABLE_NAME, &id_to_string(&id))?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    pub fn state(&self) -> TransactionState {
        unimplemented!()
    }

    pub fn in_message_id(&self) -> MessageId {
        unimplemented!()
    }

    pub fn load_in_message(&self) -> SdkResult<Box<Stream<Item = Message, Error = SdkError>>> {
        Message::load(self.in_message_id())
    }

    pub fn out_messages_id(&self) -> &Vec<MessageId> {
        unimplemented!()
    }

    pub fn id(&self) -> MessageId {
        self.id.clone()
    }

    pub fn load_out_messages(&self) -> SdkResult<Box<Stream<Item = Message, Error = SdkError>>> {
        let mut msg_id_iter = self.out_messages_id().iter();
        if let Some(id) = msg_id_iter.next().clone() {
            let mut stream = Message::load(id.clone())?;
            for id in msg_id_iter {
                stream = Box::new(stream.chain(Message::load(id.clone())?));
            }
            Ok(stream)
        } else {
            // TODO how to return empty Stream?
            bail!(SdkErrorKind::NoData);
        }
    }
}
