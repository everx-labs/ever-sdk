use super::KeyPair;
use super::Error;
use crate::client::ClientContext;
use crate::error::ClientResult;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct SigningBoxHandle(pub u32);

impl From<u32> for SigningBoxHandle {
    fn from(handle: u32) -> Self {
        Self(handle)
    }
}

#[async_trait::async_trait]
pub trait SigningBox {
    /// Get public key of key pair
    async fn get_public_key(&self) -> ClientResult<Vec<u8>>;
    /// Sign data with key pair
    async fn sign(&self, unsigned: &[u8]) -> ClientResult<Vec<u8>>;
}

pub(crate) struct KeysSigningBox {
    key_pair: ed25519_dalek::Keypair
}

impl KeysSigningBox {
    pub fn new(key_pair: ed25519_dalek::Keypair) -> Self {
        Self {
            key_pair
        }
    }

    pub fn from_encoded(key_pair: KeyPair) -> ClientResult<Self> {
        key_pair.decode().map(|pair| Self::new(pair))
    }
}

#[async_trait::async_trait]
impl SigningBox for KeysSigningBox {
    async fn get_public_key(&self) -> ClientResult<Vec<u8>> {
        Ok(self.key_pair.public.to_bytes().to_vec())
    }

    async fn sign(&self, unsigned: &[u8]) -> ClientResult<Vec<u8>> {
        super::internal::sign_using_keys(unsigned, &self.key_pair).map(|result| result.1)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct ResultOfGetSigningBox {
    /// Handle of the signing box.
    pub handle: SigningBoxHandle,
}

/// Gets a default signing box implementation.
#[api_function]
pub async fn get_signing_box(
    context: std::sync::Arc<ClientContext>,
    params: KeyPair,
) -> ClientResult<ResultOfGetSigningBox> {
    let id = context.get_next_id();
    let signing_box = KeysSigningBox::from_encoded(params)?;
    context.boxes.signing_boxes.insert(id, Box::new(signing_box));

    Ok(ResultOfGetSigningBox {
        handle: SigningBoxHandle(id),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct ResultOfRegisterSigningBox {
    /// Handle of the signing box.
    pub handle: SigningBoxHandle,
}

/// Register an application implemented signing box.
pub async fn register_signing_box(
    context: std::sync::Arc<ClientContext>,
    signing_box: impl SigningBox + Send + Sync + 'static,
) -> ClientResult<ResultOfRegisterSigningBox> {
    let id = context.get_next_id();
    context.boxes.signing_boxes.insert(id, Box::new(signing_box));

    Ok(ResultOfRegisterSigningBox {
        handle: SigningBoxHandle(id),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct ParamsOfSigningBoxGetPublicKey {
    /// Signing Box handle.
    pub signing_box: SigningBoxHandle,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct ResultOfSigningBoxGetPublicKey {
    /// Public key of signing box. Encoded with hex
    pub pubkey: String,
}

/// Returns public key of signing key pair.
#[api_function]
pub async fn signing_box_get_public_key(
    context: Arc<ClientContext>,
    params: ParamsOfSigningBoxGetPublicKey,
) -> ClientResult<ResultOfSigningBoxGetPublicKey> {
    let signing_box = context.boxes.signing_boxes
        .get(&params.signing_box.0)
        .ok_or(Error::signing_box_not_registered(params.signing_box.0))?;

    let key = signing_box.1.get_public_key().await?;

    Ok(ResultOfSigningBoxGetPublicKey {
        pubkey: hex::encode(&key)
    })
}
    
#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct ParamsOfSigningBoxSign {
    /// Signing Box handle.
    pub signing_box: SigningBoxHandle,
    /// Unsigned user data. Must be encoded with `base64`.
    pub unsigned: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct ResultOfSigningBoxSign {
    /// Data signature. Encoded with `base64`.
    pub signature: String,
}

/// Returns signed user data.
#[api_function]
pub async fn signing_box_sign(
    context: Arc<ClientContext>,
    params: ParamsOfSigningBoxSign,
) -> ClientResult<ResultOfSigningBoxSign> {
    let signing_box = context.boxes.signing_boxes
        .get(&params.signing_box.0)
        .ok_or(Error::signing_box_not_registered(params.signing_box.0))?;

    let unsigned = crate::encoding::base64_decode(&params.unsigned)?;

    let signed = signing_box.1.sign(&unsigned).await?;

    Ok(ResultOfSigningBoxSign {
        signature: hex::encode(&signed)
    })
}
