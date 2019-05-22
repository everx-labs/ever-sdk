use crate::*;
use tvm::types::UInt256;
use futures::stream::Stream;
use ton_block::{TransactionProcesingStatus, MessageId};

pub type TransactionId = UInt256;

#[derive(Debug)]
pub struct Transaction {
    tr: ton_block::Transaction,
}

const TR_TABLE_NAME: &str = "transactions";

#[allow(dead_code)]
impl Transaction {
    pub fn load(id: TransactionId) -> SdkResult<Box<Stream<Item = Transaction, Error = SdkError>>> {
        let map = db_helper::load_record(TR_TABLE_NAME, &id_to_string(&id))?
            .and_then(|val| {
                let tr: ton_block::Transaction = serde_json::from_value(val)
                    .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing transaction: {}", err)))?;

                Ok(Transaction { tr })
            });

        Ok(Box::new(map))
    }

    pub fn status(&self) -> TransactionProcesingStatus {
        self.tr.processing_status()
    }

    pub fn in_message_id(&self) -> Option<MessageId> {
        self.tr.in_message().map(|m| m.sdk_ref_unwrap().clone())
    }

    pub fn load_in_message(&self) -> SdkResult<Box<Stream<Item = Message, Error = SdkError>>> {
        match self.in_message_id() {
            Some(m) => Message::load(m),
            None => bail!(SdkErrorKind::InvalidOperation("transaction doesn't have inbound message".into()))
        }
    }

    pub fn out_messages_id(&self) -> &Vec<MessageId> {
        &self.tr.out_msgs_sdk()
    }

    pub fn id(&self) -> TransactionId {
        self.tr.id.clone()
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