use serde_json::Value;
use crate::abi::abi::Abi;
use crate::error::ApiResult;

pub enum MessageType {
    FunctionInput,
    FunctionOutput,
    Event,
}

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfDecodeMessage {
    pub message_type: MessageType,
    pub name: String,
    pub values: Value,
}

//---------------------------------------------------------------------------------- decode_message

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfDecodeMessage {
    /// contract ABI
    pub abi: Abi,

    /// Message BOC
    pub message: String,
}

pub fn decode_message(params: ParamsOfDecodeMessage) -> ApiResult<ResultOfDecodeMessage> {
    Ok(ResultOfDecodeMessage {
        message_type: MessageType::FunctionInput,
        name: "".into(),
        values: Value::Null,
    })
}

