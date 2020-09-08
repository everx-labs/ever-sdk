use serde_json::Value;
use crate::abi::abi::Abi;
use crate::error::ApiResult;

#[derive(Serialize, Deserialize, TypeInfo)]
pub enum MessageType {
    /// Message is an inbound function.
    /// Values is a function's inputs.
    FunctionInput,
    /// Message is a return value of function.
    /// Values is a function's outputs.
    FunctionOutput,
    /// Message is an emitted event.
    /// Values is an event parameters (inputs).
    Event,
}

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfDecodeMessage {
    /// Type of decoded message.
    pub message_type: MessageType,
    /// Function or event name.
    pub name: String,
    /// Input or output values (depends on `message_type`).
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

pub fn decode_message(_params: ParamsOfDecodeMessage) -> ApiResult<ResultOfDecodeMessage> {
    Ok(ResultOfDecodeMessage {
        message_type: MessageType::FunctionInput,
        name: "".into(),
        values: Value::Null,
    })
}

