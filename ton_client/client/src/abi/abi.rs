use crate::abi::Error;
use crate::crypto::internal::key_encode;
use crate::error::{ApiResult};
use serde_json::Value;
use ton_abi::{Token, TokenValue};

#[derive(Serialize, Deserialize, Clone, Debug, TypeInfo)]
pub struct AbiHandle(u32);

#[derive(Serialize, Deserialize, Clone, Debug, TypeInfo)]
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
#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug, Clone, Default)]
pub struct FunctionHeader {
    /// Message expiration time in seconds.
    pub expire: Option<u32>,

    /// Message creation time in seconds.
    pub time: Option<u64>,

    /// Public key used to sign message. Encoded with `hex`.
    pub pubkey: Option<String>,
}

impl FunctionHeader {
    pub fn from(tokens: &Vec<Token>) -> ApiResult<Option<Self>> {
        if tokens.len() == 0 {
            return Ok(None);
        }
        let mut header = FunctionHeader::default();
        for token in tokens {
            match token.name.as_str() {
                "time" => {
                    header.time = Some(match token.value {
                        TokenValue::Time(v) => Ok(v),
                        _ => Err(Error::invalid_message_for_decode(
                            "`time` header has invalid format",
                        )),
                    }?)
                }
                "expire" => {
                    header.expire = Some(match token.value {
                        TokenValue::Expire(v) => Ok(v),
                        _ => Err(Error::invalid_message_for_decode(
                            "`expire` header has invalid format",
                        )),
                    }?)
                }
                "pubkey" => {
                    header.pubkey = match token.value {
                        TokenValue::PublicKey(key) => {
                            Ok(key.as_ref().map(|x| key_encode(x.as_bytes())))
                        }
                        _ => Err(Error::invalid_message_for_decode(
                            "`pubkey` header has invalid format",
                        )),
                    }?
                }
                _ => (),
            }
        }
        Ok(Some(header))
    }
}
