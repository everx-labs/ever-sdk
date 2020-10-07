use crate::abi::Error;
use crate::error::ApiResult;
use serde_json::Value;
use ton_abi::{Token, TokenValue};

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct AbiHandle(u32);

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub enum Abi {
    Serialized(Value),
    Handle(AbiHandle),
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

fn required_time(token: &Token) -> ApiResult<u64> {
    match &token.value {
        TokenValue::Time(v) => Ok(v.clone()),
        _ => Err(Error::invalid_message_for_decode(
            "`time` header has invalid format",
        )),
    }
}

fn required_expire(token: &Token) -> ApiResult<u32> {
    match &token.value {
        TokenValue::Expire(v) => Ok(v.clone()),
        _ => Err(Error::invalid_message_for_decode(
            "`expire` header has invalid format",
        )),
    }
}

fn required_pubkey(token: &Token) -> ApiResult<Option<String>> {
    match token.value {
        TokenValue::PublicKey(key) => Ok(key.as_ref().map(|x| hex::encode(x.as_bytes()))),
        _ => Err(Error::invalid_message_for_decode(
            "`pubkey` header has invalid format",
        )),
    }
}

impl FunctionHeader {
    pub fn from(tokens: &Vec<Token>) -> ApiResult<Option<Self>> {
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
