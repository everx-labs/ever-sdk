use crate::abi::abi::Abi;
use crate::abi::internal::resolve_abi;
use crate::client::ClientContext;
use crate::encoding::base64_decode;
use crate::error::{ApiError, ApiResult};
use serde_json::Value;
use std::sync::Arc;
use ton_abi::contract::DecodedMessage;
use ton_abi::token::Detokenizer;
use ton_sdk::AbiContract;

#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug)]
pub enum DecodedMessageBody {
    /// Message has no body.
    Empty,
    /// Message body format is not ABI compliant.
    Unknown,
    /// Message is an inbound function.
    FunctionInput(
        /// Function name.
        String,
        /// Function input.
        Value,
    ),
    /// Message is a return value of function.
    FunctionOutput(
        /// Function name.
        String,
        /// Function output.
        Value,
    ),
    /// Message is an emitted event.
    Event(
        /// Event name.
        String,
        /// Event parameters.
        Value,
    ),
}

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfDecodeMessage {
    /// Decoded message body.
    pub body: DecodedMessageBody,
}

//---------------------------------------------------------------------------------- decode_message

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfDecodeMessage {
    /// contract ABI
    pub abi: Abi,

    /// Message BOC
    pub message: String,
}

use DecodedMessageBody::*;
pub fn decode_message(
    _context: Arc<ClientContext>,
    params: ParamsOfDecodeMessage,
) -> ApiResult<ResultOfDecodeMessage> {
    let (abi, message) = prepare_decode(&params)?;
    let body = if let Some(body) = message.body() {
        if let Ok(output) = abi.decode_output(body.clone(), message.is_internal()) {
            let values = get_values(&output)?;
            if abi.events().get(&output.function_name).is_some() {
                Event(output.function_name, values)
            } else {
                FunctionOutput(output.function_name, values)
            }
        } else if let Ok(input) = abi.decode_input(body.clone(), message.is_internal()) {
            let values = get_values(&input)?;
            FunctionInput(input.function_name, values)
        } else {
            Unknown
        }
    } else {
        Empty
    };
    Ok(ResultOfDecodeMessage { body })
}

fn prepare_decode(params: &ParamsOfDecodeMessage) -> ApiResult<(AbiContract, ton_block::Message)> {
    let abi = resolve_abi(&params.abi)?;
    let abi = AbiContract::load(abi.as_bytes()).map_err(|x| ApiError::abi_invalid_json(x))?;
    let message = ton_sdk::Contract::deserialize_message(&base64_decode(&params.message)?)
        .map_err(|x| ApiError::abi_invalid_message_for_decode(x))?;
    Ok((abi, message))
}

fn get_values(decoded: &DecodedMessage) -> ApiResult<Value> {
    Ok(
        Detokenizer::detokenize_to_json_value(&decoded.params, &decoded.tokens)
            .map_err(|x| ApiError::abi_invalid_message_for_decode(x))?,
    )
}
