use crate::crypto::{KeyPair, SigningBoxHandle};
use crate::error::{ApiError, ApiResult};

#[derive(Serialize, Deserialize, Clone, Debug, TypeInfo)]
pub enum MessageSigning {
    /// Message mustn't be signed.
    None,
    /// Message will be signed using external methods.
    /// Public key must be provided with `hex` encoding.
    External(String),
    /// Message will be signed using the provided keys.
    Keys(KeyPair),
    /// Message will be signed using the provided signing box.
    Box(SigningBoxHandle),
}

impl MessageSigning {
    pub fn resolve_keys(&self) -> ApiResult<Option<KeyPair>> {
        match self {
            MessageSigning::Keys(keys) => Ok(Some(keys.clone())),
            MessageSigning::Box(_) => Err(ApiError::contracts_create_deploy_message_failed(
                "Abi handle doesn't supported yet",
            )),
            MessageSigning::None => Ok(None),
            MessageSigning::External(_) => Ok(None),
        }
    }

    pub fn resolve_public_key(&self) -> ApiResult<Option<String>> {
        match self {
            MessageSigning::Keys(keys) => Ok(Some(keys.public.clone())),
            MessageSigning::Box(_) => Err(ApiError::contracts_create_deploy_message_failed(
                "Abi handle doesn't supported yet",
            )),
            MessageSigning::None => Ok(None),
            MessageSigning::External(public_key) => Ok(Some(public_key.clone())),
        }
    }
}
