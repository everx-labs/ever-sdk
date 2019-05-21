use crate::*;
use tvm::types::UInt256;
use futures::stream::Stream;

pub type TransactionId = UInt256;

// TODO this enum should be imported from ton_node module
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum TransactionProcesingStatus {
    Unknown,
    Proposed,
    Finalized,
    Refused,
}

#[derive(Debug)]
pub struct Transaction {
    id: TransactionId,
    in_msg: MessageId,
    out_msgs: Vec<MessageId>,
    status: TransactionProcesingStatus,
}

const TR_TABLE_NAME: &str = "transactions";

#[allow(dead_code)]
impl Transaction {
    pub fn load(id: TransactionId) -> SdkResult<Box<Stream<Item = Transaction, Error = SdkError>>> {
        let map = db_helper::load_record(TR_TABLE_NAME, &id_to_string(&id))?
            .map(move |val| {
                Self::parse_json(val).expect("error parsing Transaction") // TODO process error
            });

        Ok(Box::new(map))
    }

    fn parse_json(val: serde_json::Value) -> SdkResult<Transaction> {

        let id: TransactionId = hex::decode(val["id"].as_str().unwrap()).unwrap().into();
        let in_msg: MessageId = hex::decode(val["in_message"].as_str().unwrap()).unwrap().into();

        let s = format!("\"{}\"", val["status"].as_str().unwrap());
        let status: TransactionProcesingStatus = serde_json::from_str(&s).unwrap();
        
        let mut out_msgs = Vec::<MessageId>::new();
        for msg_id in val["out_messages"].as_array().unwrap() {
            out_msgs.push(hex::decode(msg_id.as_str().unwrap()).unwrap().into())
        }

        Ok(Transaction { id, in_msg, out_msgs, status })
    }

    pub fn load_json(id: TransactionId) -> SdkResult<Box<Stream<Item = String, Error = SdkError>>> {

        let map = db_helper::load_record(TR_TABLE_NAME, &id_to_string(&id))?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    pub fn state(&self) -> TransactionProcesingStatus {
        self.status.clone()
    }

    pub fn in_message_id(&self) -> MessageId {
        self.in_msg.clone()
    }

    pub fn load_in_message(&self) -> SdkResult<Box<Stream<Item = Message, Error = SdkError>>> {
        Message::load(self.in_message_id())
    }

    pub fn out_messages_id(&self) -> &Vec<MessageId> {
        &self.out_msgs
    }

    pub fn id(&self) -> TransactionId {
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

mod tests {

    #[test]
    fn test_parse() {
        let js = r#"{
            "block" :null,
            "id": "21a0b2ea5396236e86eff6529eb89eee82653bd12421b8a10ff9c7abec2ec078",
            "in_message": "21a0b2ea5396236e86eff6529eb89eee82653bd12421b8a10ff9c7abec2ec078",
            "out_messages": ["21a0b2ea5396236e86eff6509eb89eee82653bd12421b8a10ff9c7abec2ec078", "21a0b2ea5396236e86eff6511eb89eee82653bd12421b8a10ff9c7abec2ec078"],
            "status": "Proposed"}"#;
        let tr = super::Transaction::parse_json(serde_json::from_str(js).unwrap()).unwrap();
        println!("{:?}", tr);
    }
}