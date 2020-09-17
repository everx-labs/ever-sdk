use crate::abi::abi::Abi;
use crate::abi::internal::resolve_abi;
use crate::abi::Error;
use crate::client::ClientContext;
use crate::encoding::base64_decode;
use crate::error::ApiResult;
use serde_json::Value;
use std::sync::Arc;
use ton_abi::contract::DecodedMessage;
use ton_abi::token::Detokenizer;
use ton_sdk::AbiContract;

#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug)]
pub enum MessageContentType {
    /// Message contains a function parameters.
    FunctionInput,
    /// Message contains a return value of function.
    FunctionOutput,
    /// Message contains an event parameters.
    Event,
}

#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug)]
pub struct ResultOfDecodeMessage {
    /// Type of the message body content.
    pub content_type: MessageContentType,
    /// Function or event name.
    pub name: String,
    /// Parameters or result value.
    pub value: Value,
}

impl ResultOfDecodeMessage {
    fn new(content_type: MessageContentType, decoded: DecodedMessage) -> ApiResult<Self> {
        let value = Detokenizer::detokenize_to_json_value(&decoded.params, &decoded.tokens)
            .map_err(|x| Error::invalid_message_for_decode(x))?;
        Ok(Self {
            content_type,
            name: decoded.function_name,
            value,
        })
    }
}
//---------------------------------------------------------------------------------- decode_message

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfDecodeMessage {
    /// contract ABI
    pub abi: Abi,

    /// Message BOC
    pub message: String,
}

pub fn decode_message(
    _context: Arc<ClientContext>,
    params: ParamsOfDecodeMessage,
) -> ApiResult<ResultOfDecodeMessage> {
    let (abi, message) = prepare_decode(&params)?;
    if let Some(body) = message.body() {
        if let Ok(output) = abi.decode_output(body.clone(), message.is_internal()) {
            if abi.events().get(&output.function_name).is_some() {
                ResultOfDecodeMessage::new(MessageContentType::Event, output)
            } else {
                ResultOfDecodeMessage::new(MessageContentType::FunctionOutput, output)
            }
        } else if let Ok(input) = abi.decode_input(body.clone(), message.is_internal()) {
            ResultOfDecodeMessage::new(MessageContentType::FunctionInput, input)
        } else {
            Err(Error::invalid_message_for_decode(
                "The message body does not match the specified ABI",
            ))
        }
    } else {
        Err(Error::invalid_message_for_decode(
            "The message body is empty",
        ))
    }
}

fn prepare_decode(params: &ParamsOfDecodeMessage) -> ApiResult<(AbiContract, ton_block::Message)> {
    let abi = resolve_abi(&params.abi)?;
    let abi = AbiContract::load(abi.as_bytes()).map_err(|x| Error::invalid_json(x))?;
    let message = ton_sdk::Contract::deserialize_message(&base64_decode(&params.message)?)
        .map_err(|x| Error::invalid_message_for_decode(x))?;
    Ok((abi, message))
}
