use tvm::stack::{SliceData, CellData};
use tvm::block::{
    CommonMsgInfo, Message as TvmMessage, GenericId
};
use std::sync::Arc;

#[cfg(feature = "node_interaction")]
use crate::*;
#[cfg(feature = "node_interaction")]
use futures::stream::Stream;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub enum MessageType {
    Internal = 0,
    ExternalInbound = 1,
    ExternalOutbound = 2,
    Unknown = 0xff,
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Unknown
    }
}

pub type MessageId = StringId;

#[derive(Debug, Deserialize, Default)]
pub struct Message {
    pub id: MessageId,
    #[serde(deserialize_with = "json_helper::deserialize_tree_of_cells_opt_cell")]
    pub body: Option<Arc<CellData>>,
    pub msg_type: MessageType,
    pub transaction_id: Option<TransactionId>,
}

#[cfg(feature = "node_interaction")]
const MESSAGE_FIELDS: &str = r#"
    id
    body
    msg_type
    transaction_id
"#;

// The struct represents sent message and allows to access their properties.
#[allow(dead_code)]
impl Message {

    // Asynchronously loads a Message instance or None if message with given id is not exists
    #[cfg(feature = "node_interaction")]
    pub fn load(id: &MessageId) -> SdkResult<Box<dyn Stream<Item = Option<Message>, Error = SdkError>>> {
        let map = queries_helper::load_record_fields(
            MESSAGES_TABLE_NAME,
            &id.to_string(),
            MESSAGE_FIELDS
            )?
                .and_then(|val| {
                    if val == serde_json::Value::Null {
                        Ok(None)
                    } else {
                        let msg: Message = serde_json::from_value(val)
                            .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing message: {}", err)))?;

                        Ok(Some(msg))
                    }
            });

        Ok(Box::new(map))
    }

    // Asynchronously loads a Message's json representation 
    // or null if message with given id is not exists
    #[cfg(feature = "node_interaction")]
    pub fn load_json(id: MessageId) -> SdkResult<Box<dyn Stream<Item = String, Error = SdkError>>> {

        let map = queries_helper::load_record_fields(
            MESSAGES_TABLE_NAME,
            &id.to_string(),
            MESSAGE_FIELDS
            )?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    pub fn with_msg(tvm_msg: TvmMessage) -> SdkResult<Self> {
        let mut msg = Self::default();
        msg.id = tvm_msg.calc_id()?.as_slice()[..].into();
        msg.body = tvm_msg.body().map(|slice| slice.cell().clone());

        msg.msg_type = match tvm_msg.header() {
            CommonMsgInfo::IntMsgInfo(_) => MessageType::Internal,
            CommonMsgInfo::ExtInMsgInfo(_) => MessageType::ExternalInbound,
            CommonMsgInfo::ExtOutMsgInfo(_) => MessageType::ExternalOutbound
        };

        Ok(msg)
    }

    // Returns message's identifier
    pub fn id(&self) -> MessageId {
        // On client side id is ready allways. It is never be calculated, just returned.
        self.id.clone()
    }

    // Returns message's body (as tree of cells) or None if message doesn't have once
    pub fn body(&self) -> Option<SliceData> {
        self.body.clone().map(|cell| cell.into())
    }

    // Returns message's type
    pub fn msg_type(&self) -> MessageType {
        self.msg_type.clone()
    }
}