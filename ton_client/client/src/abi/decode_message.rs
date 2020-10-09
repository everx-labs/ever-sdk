use crate::abi::types::Abi;
use crate::abi::internal::resolve_abi;
use crate::abi::{Error, FunctionHeader};
use crate::client::ClientContext;
use crate::encoding::base64_decode;
use crate::error::ClientResult;
use serde_json::Value;
use std::sync::Arc;
use ton_abi::contract::DecodedMessage;
use ton_abi::token::Detokenizer;
use ton_sdk::AbiContract;

#[derive(Serialize, Deserialize, ApiType, PartialEq, Debug, Clone)]
pub enum DecodedMessageType {
    /// Message contains the input of the ABI function.
    FunctionInput,

    /// Message contains the output of the ABI function.
    FunctionOutput,

    /// Message contains the input of the foreign ABI function.
    ///
    /// Occurs when contract sends internal message to other
    /// contract.
    ForeignFunctionInput,

    /// Message contains the input of the ABI event.
    Event,
}

#[derive(Serialize, Deserialize, ApiType, PartialEq, Debug, Clone)]
pub struct DecodedMessageBody {
    /// Type of the message body content.
    pub message_type: DecodedMessageType,

    /// Function or event name.
    ///
    /// In case of foreign function input the name contains a comma
    /// separated list of possible fully qualified names, for
    /// example: "IFoo.foo,IBar.foo".
    pub name: String,

    /// Parameters or result value.
    pub value: Value,

    /// Function header.
    pub header: Option<FunctionHeader>,
}

impl DecodedMessageBody {
    fn new(
        message_type: DecodedMessageType,
        decoded: DecodedMessage,
        header: Option<FunctionHeader>,
    ) -> ClientResult<Self> {
        let value = Detokenizer::detokenize_to_json_value(&decoded.params, &decoded.tokens)
            .map_err(|x| Error::invalid_message_for_decode(x))?;
        Ok(Self {
            message_type,
            name: decoded.function_name,
            value,
            header,
        })
    }
}
//---------------------------------------------------------------------------------- decode_message

#[derive(Serialize, Deserialize, ApiType)]
pub struct ParamsOfDecodeMessage {
    /// contract ABI
    pub abi: Abi,

    /// Message BOC
    pub message: String,
}

#[api_function]
pub fn decode_message(
    _context: Arc<ClientContext>,
    params: ParamsOfDecodeMessage,
) -> ClientResult<DecodedMessageBody> {
    let (abi, message) = prepare_decode(&params)?;
    if let Some(body) = message.body() {
        if let Ok(output) = abi.decode_output(body.clone(), message.is_internal()) {
            if abi.events().get(&output.function_name).is_some() {
                DecodedMessageBody::new(DecodedMessageType::Event, output, None)
            } else {
                DecodedMessageBody::new(DecodedMessageType::FunctionOutput, output, None)
            }
        } else if let Ok(input) = abi.decode_input(body.clone(), message.is_internal()) {
            // TODO: add pub access to `abi_version` field of `Contract` struct.
            let abi_version = abi
                .functions()
                .values()
                .next()
                .map(|x| x.abi_version)
                .unwrap_or(1);
            let (header, _, _) = ton_abi::Function::decode_header(
                abi_version,
                body.clone(),
                abi.header(),
                message.is_internal(),
            )
            .map_err(|err| {
                Error::invalid_message_for_decode(format!("Can't decode function header: {}", err))
            })?;
            DecodedMessageBody::new(
                DecodedMessageType::FunctionInput,
                input,
                FunctionHeader::from(&header)?,
            )
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

fn prepare_decode(params: &ParamsOfDecodeMessage) -> ClientResult<(AbiContract, ton_block::Message)> {
    let abi = resolve_abi(&params.abi)?;
    let abi = AbiContract::load(abi.as_bytes()).map_err(|x| Error::invalid_json(x))?;
    let message = ton_sdk::Contract::deserialize_message(&base64_decode(&params.message)?)
        .map_err(|x| Error::invalid_message_for_decode(x))?;
    Ok((abi, message))
}
