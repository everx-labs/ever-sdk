use crate::abi::Error;
use crate::client;
use crate::crypto::{KeyPair, SigningBoxHandle};
use crate::error::ClientResult;

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
#[serde(tag="type")]
pub enum Signer {
    /// Message mustn't be signed.
    None,
    /// Message will be signed using external methods.
    /// Public key must be provided in `hex` encoding.
    External { public_key: String },
    /// Message will be signed using the provided keys.
    Keys { keys: KeyPair },
    /// Message will be signed using the provided signing box.
    SigningBox { handle: SigningBoxHandle },
}

impl Signer {
    pub(crate) fn is_external(&self) -> bool {
        if let Signer::External { .. } = self {
            true
        } else {
            false
        }
    }
}

impl Signer {
    pub fn resolve_keys(&self) -> ClientResult<Option<KeyPair>> {
        match self {
            Signer::None => Ok(None),
            Signer::Keys { keys } => Ok(Some(keys.clone())),
            Signer::External { .. } => Ok(None),
            Signer::SigningBox { .. } => Err(Error::invalid_signer(
                "Signing box can't provide secret key".into(),
            )),
        }
    }

    pub fn resolve_public_key(&self) -> ClientResult<Option<String>> {
        match self {
            Signer::None => Ok(None),
            Signer::Keys { keys } => Ok(Some(keys.public.clone())),
            Signer::External { public_key } => Ok(Some(public_key.clone())),
            Signer::SigningBox { .. } => Err(client::Error::not_implemented(
                "Signing boxes doesn't supported yet",
            )),
        }
    }
}
