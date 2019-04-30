use crate::*;
use tvm::types::UInt256;
use futures::stream::Stream;
use std::sync::Arc;
use tvm::stack::CellData;
use tvm::cells_serialization::{deserialize_cells_tree};
use std::io::Cursor;

pub type MessageId = UInt256;

// TODO this enum should be imported from ton_node module
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum MessageState {
    Unknown,
    Queued,
    Processing,
    Proposed,
    Finalized,
    Refused,
}


// TODO this enum should be imported from ton_node module
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum MessageType {
    Unknown,
    Internal,
    ExternalInbound,
    ExternalOutbound
}

// TODO need to realise to_string (or something else) for UInt256 in node
pub fn id_to_string(id: &UInt256) -> String {
    hex::encode(id.as_slice())
}

const MSG_TABLE_NAME: &str = "messages";

#[derive(Debug)]
pub struct Message {
    id: MessageId,
    body: Arc<CellData>,
    msg_type: MessageType
}

#[allow(dead_code)]
impl Message {

    pub fn load(id: MessageId) -> SdkResult<Box<Stream<Item = Message, Error = SdkError>>> {
        let map = db_helper::load_record(MSG_TABLE_NAME, &id_to_string(&id))?
            .map(move |val| {
                Self::parse_json(val).expect("error parsing Message") // TODO process error
            });

        Ok(Box::new(map))
    }

    fn parse_json(val: serde_json::Value) -> SdkResult<Message> {

        let msg_type = match val["type"].as_str().unwrap() {
            "internal" => MessageType::Internal,
            "extarnal inbound" => MessageType::ExternalInbound,
            "external outbound" => MessageType::ExternalOutbound,
            _ => MessageType::Unknown,
        }.into();
        
        let id: TransactionId = hex::decode(val["id"].as_str().unwrap()).unwrap().into();
        let raw_body = base64::decode(val["body"].as_str().unwrap()).unwrap(); //hex::decode(val["in_message"].as_str().unwrap()).unwrap();

        let mut raw_body = Cursor::new(raw_body);
        let mut body_roots = deserialize_cells_tree(&mut raw_body)?;
        //if body_roots.len() != 1 {
        //    bail!(SdkErrorKind::InvalidData("Invalid bag of cells".into()));
        //}
        let body = body_roots.remove(0);

        Ok(Message { id, body, msg_type })
    }

    pub fn load_json(id: MessageId) -> SdkResult<Box<Stream<Item = String, Error = SdkError>>> {

        let map = db_helper::load_record(MSG_TABLE_NAME, &id_to_string(&id))?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    pub fn msg_type(&self) -> MessageType {
        self.msg_type.clone()
    }

    pub fn state(&self) -> MessageState {
        unimplemented!()
    }

    pub fn id(&self) -> MessageId {
        self.id.clone()
    }

    pub fn body(&self) -> Arc<CellData> {
        self.body.clone()
    } 
}


mod tests {
    
    use super::*;

    #[test]
    fn test_parse() {
        let js = r#"
         {"id":"0000000000000000000000000000000000000000000000000000000000000000","type":"extarnal inbound","status":"Proposed","block":null,"info":{"source":null,"destination":{"type":"internal_std","anycast":null,"workchain_id":-1,"address":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"},"import_fee":0},"state_init":null,"body":"te6ccoEBAQEABgAGAAgAAAAD"}
            "#;
        let m = Message::parse_json(serde_json::from_str(js).unwrap()).unwrap();
        println!("{:?}", m);
    }
}