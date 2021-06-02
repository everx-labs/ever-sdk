use crate::{abi::types::Abi, boc::internal::deserialize_cell_from_boc};
use crate::abi::{Error, FunctionHeader};
use crate::boc::internal::deserialize_object_from_boc;
use crate::client::ClientContext;
use crate::error::ClientResult;
use serde_json::Value;
use std::sync::Arc;
use ton_abi::contract::DecodedMessage;
use ton_abi::token::Detokenizer;
use ton_sdk::AbiContract;
use ton_types::SliceData;

#[derive(Serialize, Deserialize, ApiType, PartialEq, Debug, Clone)]
pub enum MessageBodyType {
    /// Message contains the input of the ABI function.
    Input,

    /// Message contains the output of the ABI function.
    Output,

    /// Message contains the input of the imported ABI function.
    ///
    /// Occurs when contract sends an internal message to other
    /// contract.
    InternalOutput,

    /// Message contains the input of the ABI event.
    Event,
}

#[derive(Serialize, Deserialize, ApiType, PartialEq, Debug, Clone)]
pub struct DecodedMessageBody {
    /// Type of the message body content.
    pub body_type: MessageBodyType,

    /// Function or event name.
    pub name: String,

    /// Parameters or result value.
    pub value: Option<Value>,

    /// Function header.
    pub header: Option<FunctionHeader>,
}

impl DecodedMessageBody {
    fn new(
        body_type: MessageBodyType,
        decoded: DecodedMessage,
        header: Option<FunctionHeader>,
    ) -> ClientResult<Self> {
        let value = Detokenizer::detokenize_to_json_value(&decoded.params, &decoded.tokens)
            .map_err(|x| Error::invalid_message_for_decode(x))?;
        Ok(Self {
            body_type,
            name: decoded.function_name,
            value: Some(value),
            header,
        })
    }
}

//---------------------------------------------------------------------------------- decode_message

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfDecodeMessage {
    /// contract ABI
    pub abi: Abi,

    /// Message BOC
    pub message: String,
}

/// Decodes message body using provided message BOC and ABI.
#[api_function]
pub async fn decode_message(
    context: Arc<ClientContext>,
    params: ParamsOfDecodeMessage,
) -> ClientResult<DecodedMessageBody> {
    let (abi, message) = prepare_decode(&context, &params).await?;
    if let Some(body) = message.body() {
        decode_body(abi, body, message.is_internal())
    } else {
        Err(Error::invalid_message_for_decode(
            "The message body is empty",
        ))
    }
}

//----------------------------------------------------------------------------- decode_message_body

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfDecodeMessageBody {
    /// Contract ABI used to decode.
    pub abi: Abi,

    /// Message body BOC encoded in `base64`.
    pub body: String,

    /// True if the body belongs to the internal message.
    pub is_internal: bool,
}

/// Decodes message body using provided body BOC and ABI.
#[api_function]
pub async fn decode_message_body(
    context: Arc<ClientContext>,
    params: ParamsOfDecodeMessageBody,
) -> ClientResult<DecodedMessageBody> {
    let abi = params.abi.json_string()?;
    let abi = AbiContract::load(abi.as_bytes()).map_err(|x| Error::invalid_json(x))?;
    let (_, body) = deserialize_cell_from_boc(&context, &params.body, "message body").await?;
    decode_body(abi, body.into(), params.is_internal)
}

async fn prepare_decode(
    context: &ClientContext,
    params: &ParamsOfDecodeMessage,
) -> ClientResult<(AbiContract, ton_block::Message)> {
    let abi = params.abi.json_string()?;
    let abi = AbiContract::load(abi.as_bytes()).map_err(|x| Error::invalid_json(x))?;
    let message = deserialize_object_from_boc(context, &params.message, "message")
        .await
        .map_err(|x| Error::invalid_message_for_decode(x))?;
    Ok((abi, message.object))
}

fn decode_body(
    abi: AbiContract,
    body: SliceData,
    is_internal: bool,
) -> ClientResult<DecodedMessageBody> {
    if let Ok(output) = abi.decode_output(body.clone(), is_internal) {
        if abi.events().get(&output.function_name).is_some() {
            DecodedMessageBody::new(MessageBodyType::Event, output, None)
        } else {
            DecodedMessageBody::new(MessageBodyType::Output, output, None)
        }
    } else if let Ok(input) = abi.decode_input(body.clone(), is_internal) {
        // TODO: add pub access to `abi_version` field of `Contract` struct.
        let abi_version = abi
            .functions()
            .values()
            .next()
            .map(|x| x.abi_version)
            .unwrap_or(1);
        let (header, _, _) =
            ton_abi::Function::decode_header(abi_version, body.clone(), abi.header(), is_internal)
                .map_err(|err| {
                    Error::invalid_message_for_decode(format!(
                        "Can't decode function header: {}",
                        err
                    ))
                })?;
        DecodedMessageBody::new(
            MessageBodyType::Input,
            input,
            FunctionHeader::from(&header)?,
        )
    } else {
        Err(Error::invalid_message_for_decode(
            "The message body does not match the specified ABI.\n
                Tip: Please check that you specified message's body, not full BoC.",
        ))
    }
}
