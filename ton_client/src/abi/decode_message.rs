use crate::encoding::{decode_abi_number, slice_from_cell};
use crate::{abi::types::Abi, boc::internal::deserialize_cell_from_boc};
use crate::abi::{Error, FunctionHeader};
use crate::boc::internal::deserialize_object_from_boc;
use crate::client::ClientContext;
use crate::error::ClientResult;
use serde_json::Value;
use std::sync::Arc;
use ever_abi::contract::DecodedMessage;
use ever_abi::token::Detokenizer;
use ton_sdk::{AbiContract, AbiFunction, AbiEvent};
use ever_block::SliceData;

use super::types::extend_data_to_sign;

#[derive(Serialize, Deserialize, ApiType, PartialEq, Debug, Clone)]
pub enum DataLayout {
    /// Decode message body as function input parameters.
    Input,

    /// Decode message body as function output.
    Output,
}

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
        Self::new_with_original_error(body_type, decoded, header)
            .map_err(|x| Error::invalid_message_for_decode(x))
    }

    fn new_with_original_error(
        body_type: MessageBodyType,
        decoded: DecodedMessage,
        header: Option<FunctionHeader>,
    ) -> ever_block::Result<Self> {
        let value = Detokenizer::detokenize_to_json_value(&decoded.tokens)?;
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

    /// Flag allowing partial BOC decoding when ABI doesn't describe the full body BOC.
    /// Controls decoder behaviour when after decoding all described in ABI params there are some data left in BOC:
    /// `true` - return decoded values
    /// `false` - return error of incomplete BOC deserialization (default)
    #[serde(default)]
    pub allow_partial: bool,

    /// Function name or function id if is known in advance
    pub function_name: Option<String>,

    // For external (inbound and outbound) messages data_layout parameter is ignored.
    // For internal: by default SDK tries to decode as output and then if decode is not successfull - tries as input.
    // If explicitly specified then tries only the specified layout.
    pub data_layout: Option<DataLayout>,
}

/// Decodes message body using provided message BOC and ABI.
#[api_function]
pub fn decode_message(
    context: Arc<ClientContext>,
    params: ParamsOfDecodeMessage,
) -> ClientResult<DecodedMessageBody> {
    let (abi, message) = prepare_decode(&context, &params)?;
    if let Some(body) = message.body() {
        let data_layout = match message.header() {
            ever_block::CommonMsgInfo::ExtInMsgInfo(_) => Some(DataLayout::Input),
            ever_block::CommonMsgInfo::ExtOutMsgInfo(_) => Some(DataLayout::Output),
            ever_block::CommonMsgInfo::IntMsgInfo(_) => params.data_layout,
        };
        decode_body(abi, body, message.is_internal(), params.allow_partial, params.function_name, data_layout)
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

    /// Flag allowing partial BOC decoding when ABI doesn't describe the full body BOC.
    /// Controls decoder behaviour when after decoding all described in ABI params there are some data left in BOC:
    /// `true` - return decoded values
    /// `false` - return error of incomplete BOC deserialization (default)
    #[serde(default)]
    pub allow_partial: bool,

    /// Function name or function id if is known in advance
    pub function_name: Option<String>,

    // By default SDK tries to decode as output and then if decode is not successfull - tries as input.
	// If explicitly specified then tries only the specified layout.
    pub data_layout: Option<DataLayout>,
}

/// Decodes message body using provided body BOC and ABI.
#[api_function]
pub fn decode_message_body(
    context: Arc<ClientContext>,
    params: ParamsOfDecodeMessageBody,
) -> ClientResult<DecodedMessageBody> {
    let abi = params.abi.abi()?;
    let (_, body) = deserialize_cell_from_boc(&context, &params.body, "message body")?;
    let body = slice_from_cell(body)?;
    decode_body(abi, body, params.is_internal, params.allow_partial, params.function_name, params.data_layout)
}

fn prepare_decode(
    context: &ClientContext,
    params: &ParamsOfDecodeMessage,
) -> ClientResult<(AbiContract, ever_block::Message)> {
    let abi = params.abi.abi()?;
    let message = deserialize_object_from_boc(context, &params.message, "message")
        .map_err(|x| Error::invalid_message_for_decode(x))?;
    Ok((abi, message.object))
}

fn decode_body(
    abi: AbiContract,
    body: SliceData,
    is_internal: bool,
    allow_partial: bool,
    function_name: Option<String>,
    data_layout: Option<DataLayout>,
) -> ClientResult<DecodedMessageBody> {
    if let Some(function) = function_name {
        decode_with_function(abi, body, is_internal, allow_partial, function, data_layout)
            .map_err(|err| Error::invalid_message_for_decode(err))
    } else {
        decode_unknown_function(abi, body, is_internal, allow_partial, data_layout)
    }
}

const ERROR_TIP: &str = "The message body does not match the specified ABI. Tip: Please check that you specified message's body, not full BOC.";

fn decode_unknown_function(
    abi: AbiContract,
    body: SliceData,
    is_internal: bool,
    allow_partial: bool,
    data_layout: Option<DataLayout>,
) -> ClientResult<DecodedMessageBody> {
    let decode_output = || {
        let output = abi.decode_output(body.clone(), is_internal, allow_partial)
            .map_err(|err| Error::invalid_message_for_decode(err))?;
        if abi.events().get(&output.function_name).is_some() {
            DecodedMessageBody::new(MessageBodyType::Event, output, None)
        } else {
            DecodedMessageBody::new(MessageBodyType::Output, output, None)
        }
    };
    let decode_input = || {
        let input = abi.decode_input(body.clone(), is_internal, allow_partial)
            .map_err(|err| Error::invalid_message_for_decode(err))?;
        let (header, _, _) =
            ever_abi::Function::decode_header(abi.version(), body.clone(), abi.header(), is_internal)
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
    };
    match data_layout {
        Some(DataLayout::Input) => decode_input(),
        Some(DataLayout::Output) => decode_output(),
        None => {
            decode_output()
                .or_else(|_| decode_input())
                .or_else(|_| Err(Error::invalid_message_for_decode(ERROR_TIP)))
        }
    }
}

fn decode_with_function(
    abi: AbiContract,
    body: SliceData,
    is_internal: bool,
    allow_partial: bool,
    function_name: String,
    data_layout: Option<DataLayout>,
) -> ClientResult<DecodedMessageBody> {
    let variant = find_abi_function(&abi, &function_name)?;
    match variant {
        AbiFunctionVariant::Function(function) => {
            let decode_output = || {
                let decoded = function.decode_output(body.clone(), is_internal, allow_partial)
                    .map_err(|err| Error::invalid_message_for_decode(err))?;
                DecodedMessageBody::new(
                    MessageBodyType::Output,
                    DecodedMessage {
                        function_name: function_name.clone(),
                        tokens: decoded,
                    },
                    None
                )
            };
            let decode_input = || {
                let decoded = function.decode_input(body.clone(), is_internal, allow_partial)
                    .map_err(|err| Error::invalid_message_for_decode(err))?;
                let (header, _, _) =
                    ever_abi::Function::decode_header(abi.version(), body.clone(), abi.header(), is_internal)
                        .map_err(|err| Error::invalid_message_for_decode(err))?;
                DecodedMessageBody::new(
                    MessageBodyType::Input,
                    DecodedMessage {
                        function_name: function_name.clone(),
                        tokens: decoded,
                    },
                    FunctionHeader::from(&header)?,
                )
            };

            match data_layout {
                Some(DataLayout::Input) => decode_input(),
                Some(DataLayout::Output) => decode_output(),
                None => {
                    decode_output()
                        .or_else(|_| decode_input())
                        .or_else(|_| Err(Error::invalid_message_for_decode(ERROR_TIP)))
                }
            }
        },
        AbiFunctionVariant::Event(event) => {
            if is_internal {
                return Err(Error::invalid_message_for_decode("ABI event can be produced only in external outbound message"));
            }
            let decoded = event
                .decode_input(body, allow_partial)
                .map_err(|err| Error::invalid_message_for_decode(err))?;
            let decoded = DecodedMessage {
                function_name,
                tokens: decoded,
            };
            DecodedMessageBody::new(MessageBodyType::Event, decoded, None)
        }
    }
}

enum AbiFunctionVariant<'a> {
    Function(&'a AbiFunction),
    Event(&'a AbiEvent),
}

fn find_abi_function<'a>(abi: &'a AbiContract, name: &str) -> ClientResult<AbiFunctionVariant<'a>> {
    if let Ok(function) = abi.function(name) {
        Ok(AbiFunctionVariant::Function(function))
    } else if let Ok(event) = abi.event(name) {
        Ok(AbiFunctionVariant::Event(event))
    } else {
        let function_id: u32 = decode_abi_number(name)?;
        if let Ok(function) = abi.function_by_id(function_id, true)
            .or_else(|_| abi.function_by_id(function_id, true))
        {
            Ok(AbiFunctionVariant::Function(function))
        } else if let Ok(event) = abi.event_by_id(function_id) {
            Ok(AbiFunctionVariant::Event(event))
        } else {
            Err(Error::invalid_function_name(name))
        }
    }
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfGetSignatureData {
    /// Contract ABI used to decode.
    pub abi: Abi,

    /// Message BOC encoded in `base64`.
    pub message: String,

    /// Signature ID to be used in unsigned data preparing when CapSignatureWithId
    /// capability is enabled
    pub signature_id: Option<i32>,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfGetSignatureData {
    /// Signature from the message in `hex`.
    pub signature: String,

    /// Data to verify the signature in `base64`.
    pub unsigned: String,
}

/// Extracts signature from message body and calculates hash to verify the signature
#[api_function]
pub async fn get_signature_data(
    context: Arc<ClientContext>,
    params: ParamsOfGetSignatureData,
) -> ClientResult<ResultOfGetSignatureData> {
    let abi = params.abi.abi()?;
    let message: ever_block::Message = deserialize_object_from_boc(&context, &params.message, "message")?.object;
    if let Some(body) = message.body() {
        let address = message.dst()
            .ok_or_else(|| Error::invalid_message_for_decode(
                "Message has no destination address",
            ))?;
        let (signature, hash) = abi.get_signature_data(body, Some(address))
            .map_err(|err| Error::invalid_message_for_decode(err))?;
        let unsigned = extend_data_to_sign(&context, params.signature_id, Some(hash)).await?;
        Ok(ResultOfGetSignatureData {
            signature: hex::encode(&signature),
            unsigned: base64::encode(&unsigned.unwrap()),
        })
    } else {
        Err(Error::invalid_message_for_decode(
            "The message body is empty",
        ))
    }
}
