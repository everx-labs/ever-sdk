use crate::abi::{Error, ParamsOfEncodeMessage};
use crate::error::ClientResult;
use crate::{processing, ClientContext};
use std::sync::Arc;
use ton_abi::{Token, TokenValue};

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct AbiHandle(u32);

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
#[serde(tag = "type", content = "value")]
pub enum Abi {
    Serialized(AbiContract),
    Handle(AbiHandle),
}

impl Abi {
    pub(crate) fn json_string(&self) -> ClientResult<String> {
        match self {
            Self::Serialized(abi) => {
                Ok(serde_json::to_string(abi).map_err(|err| Error::invalid_abi(err))?)
            }
            _ => Err(crate::client::Error::not_implemented(
                "ABI handles are not supported yet",
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub struct AbiContract {
    #[serde(rename = "ABI version")]
    pub abi_version: u32,
    #[serde(default)]
    pub header: Vec<String>,
    #[serde(default)]
    pub functions: Vec<AbiFunction>,
    #[serde(default)]
    pub events: Vec<AbiEvent>,
    #[serde(default)]
    pub data: Vec<AbiData>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub struct AbiFunction {
    pub name: String,
    pub inputs: Vec<AbiParam>,
    pub outputs: Vec<AbiParam>,
    #[serde(default)]
    pub id: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub struct AbiEvent {
    pub name: String,
    pub inputs: Vec<AbiParam>,
    #[serde(default)]
    pub id: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub struct AbiData {
    pub key: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(default)]
    pub components: Vec<AbiParam>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub struct AbiParam {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(default)]
    pub components: Vec<AbiParam>,
}

/// The ABI function header.
///
/// Includes several hidden function parameters that contract
/// uses for security and replay protection reasons.
///
/// The actual set of header fields depends on the contract's ABI.
#[derive(Serialize, Deserialize, ApiType, PartialEq, Debug, Clone, Default)]
pub struct FunctionHeader {
    /// Message expiration time in seconds.
    pub expire: Option<u32>,

    /// Message creation time in milliseconds.
    pub time: Option<u64>,

    /// Public key used to sign message. Encoded with `hex`.
    pub pubkey: Option<String>,
}

fn required_time(token: &Token) -> ClientResult<u64> {
    match &token.value {
        TokenValue::Time(v) => Ok(v.clone()),
        _ => Err(Error::invalid_message_for_decode(
            "`time` header has invalid format",
        )),
    }
}

fn required_expire(token: &Token) -> ClientResult<u32> {
    match &token.value {
        TokenValue::Expire(v) => Ok(v.clone()),
        _ => Err(Error::invalid_message_for_decode(
            "`expire` header has invalid format",
        )),
    }
}

fn required_pubkey(token: &Token) -> ClientResult<Option<String>> {
    match token.value {
        TokenValue::PublicKey(key) => Ok(key.as_ref().map(|x| hex::encode(x.as_bytes()))),
        _ => Err(Error::invalid_message_for_decode(
            "`pubkey` header has invalid format",
        )),
    }
}

impl FunctionHeader {
    pub fn from(tokens: &Vec<Token>) -> ClientResult<Option<Self>> {
        if tokens.len() == 0 {
            return Ok(None);
        }
        let mut header = FunctionHeader::default();
        for token in tokens {
            match token.name.as_str() {
                "time" => header.time = Some(required_time(&token)?),
                "expire" => header.expire = Some(required_expire(&token)?),
                "pubkey" => header.pubkey = required_pubkey(&token)?,
                _ => (),
            }
        }
        Ok(Some(header))
    }
}

#[derive(Serialize, Deserialize, ApiType, Debug, Clone)]
#[serde(tag = "type")]
pub enum MessageSource {
    Encoded { message: String, abi: Option<Abi> },
    EncodingParams(ParamsOfEncodeMessage),
}

impl MessageSource {
    pub(crate) async fn encode(
        &self,
        context: &Arc<ClientContext>,
    ) -> ClientResult<(String, Option<Abi>)> {
        Ok(match self {
            MessageSource::EncodingParams(params) => {
                if params.signer.is_external() {
                    return Err(processing::Error::external_signer_must_not_be_used());
                }
                let abi = params.abi.clone();
                (
                    crate::abi::encode_message(context.clone(), params.clone())
                        .await?
                        .message,
                    Some(abi),
                )
            }
            MessageSource::Encoded { abi, message } => (message.clone(), abi.clone()),
        })
    }
}
