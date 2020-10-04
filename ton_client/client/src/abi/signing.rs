use crate::crypto::{KeyPair, SigningBoxHandle};
use crate::error::{ApiResult};
use crate::client;

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub enum Signer {
    /// Message mustn't be signed.
    None,
    /// Message will be signed using external methods.
    /// Public key must be provided with `hex` encoding.
    External(String),
    /// Message will be signed using the provided keys.
    WithKeys(KeyPair),
    /// Message will be signed using the provided signing box.
    Box(SigningBoxHandle),
}

impl Signer {
    pub fn resolve_keys(&self) -> ApiResult<Option<KeyPair>> {
        match self {
            Signer::None => Ok(None),
            Signer::WithKeys(keys) => Ok(Some(keys.clone())),
            Signer::External(_) => Ok(None),
            Signer::Box(_) => Err(client::Error::not_implemented(
                "Abi handle doesn't supported yet",
            )),
        }
    }

    pub fn resolve_public_key(&self) -> ApiResult<Option<String>> {
        match self {
            Signer::None => Ok(None),
            Signer::WithKeys(keys) => Ok(Some(keys.public.clone())),
            Signer::External(public_key) => Ok(Some(public_key.clone())),
            Signer::Box(_) => Err(client::Error::not_implemented(
                "Abi handle doesn't supported yet",
            )),
        }
    }
}
