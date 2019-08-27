use crate::*;
use tvm::stack::SliceData;
use tvm::block::{
    CommonMsgInfo, Message as TvmMessage, MessageId, MessageProcessingStatus
};

#[cfg(feature = "node_interaction")]
use futures::stream::Stream;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum MessageType {
    Unknown,
    Internal,
    ExternalInbound,
    ExternalOutbound
}

#[derive(Debug)]
pub struct Message {
    msg: TvmMessage,
}

// The struct represents sent message and allows to access their properties.
#[allow(dead_code)]
impl Message {

    // Asynchronously loads a Message instance or None if message with given id is not exists
    #[cfg(feature = "node_interaction")]
    pub fn load(id: MessageId) -> SdkResult<Box<dyn Stream<Item = Option<Message>, Error = SdkError>>> {
        let map = queries_helper::load_record(MESSAGES_TABLE_NAME, &id.to_hex_string())?
            .and_then(|val| {
                if val == serde_json::Value::Null {
                    Ok(None)
                } else {
                    let msg: TvmMessage = serde_json::from_value(val)
                        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing message: {}", err)))?;

                    Ok(Some(Message { msg }))
                }
            });

        Ok(Box::new(map))
    }

    // Asynchronously loads a Message's json representation 
    // or null if message with given id is not exists
    #[cfg(feature = "node_interaction")]
    pub fn load_json(id: MessageId) -> SdkResult<Box<dyn Stream<Item = String, Error = SdkError>>> {

        let map = queries_helper::load_record(MESSAGES_TABLE_NAME, &id.to_hex_string())?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    // Create `Message` struct with provided `TvmMessage`
    pub fn with_msg(msg: TvmMessage) -> Self {
        Message { msg }
    }

    // Returns message's processing status
    pub fn status(&self) -> MessageProcessingStatus {
        self.msg.status.clone()
    }

    // Returns message's identifier
    pub fn id(&self) -> MessageId {
        self.msg.id.clone()
    }

    // Returns message's body (as tree of cells) or None if message doesn't have once
    pub fn body(&self) -> Option<SliceData> {
        self.msg.body().into()
    }

    // Returns blockchain's message struct
    // Some node-specifed methods won't work. All TonStructVariant fields has Client variant.
    pub fn msg(&self) -> &TvmMessage {
         &self.msg
    }

    // Returns message's type
    pub fn msg_type(&self) -> MessageType {
        match self.msg.header() {
            CommonMsgInfo::IntMsgInfo(_) => MessageType::Internal,
            CommonMsgInfo::ExtInMsgInfo(_) => MessageType::ExternalInbound,
            CommonMsgInfo::ExtOutMsgInfo(_) => MessageType::ExternalOutbound,
        }
    }
}