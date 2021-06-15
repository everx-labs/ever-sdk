use std::sync::Arc;

use lockfree::map::ReadGuard;
use serde_json::Value;

use crate::client::ClientContext;
use crate::error::ClientResult;

use super::Error;
use super::KeyPair;

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
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

/// Creates a default signing box implementation.
#[api_function]
pub async fn get_signing_box(
    context: std::sync::Arc<ClientContext>,
    params: KeyPair,
) -> ClientResult<RegisteredSigningBox> {
    let id = context.get_next_id();
    let signing_box = KeysSigningBox::from_encoded(params)?;
    context.boxes.signing_boxes.insert(id, Box::new(signing_box));

    Ok(RegisteredSigningBox {
        handle: SigningBoxHandle(id),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct RegisteredSigningBox {
    /// Handle of the signing box.
    pub handle: SigningBoxHandle,
}

/// Registers an application implemented signing box.
pub async fn register_signing_box(
    context: std::sync::Arc<ClientContext>,
    signing_box: impl SigningBox + Send + Sync + 'static,
) -> ClientResult<RegisteredSigningBox> {
    let id = context.get_next_id();
    context.boxes.signing_boxes.insert(id, Box::new(signing_box));

    Ok(RegisteredSigningBox {
        handle: SigningBoxHandle(id),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ResultOfSigningBoxGetPublicKey {
    /// Public key of signing box. Encoded with hex
    pub pubkey: String,
}

/// Returns public key of signing key pair.
#[api_function]
pub async fn signing_box_get_public_key(
    context: Arc<ClientContext>,
    params: RegisteredSigningBox,
) -> ClientResult<ResultOfSigningBoxGetPublicKey> {
    let signing_box = context.boxes.signing_boxes
        .get(&params.handle.0)
        .ok_or(Error::signing_box_not_registered(params.handle.0))?;

    let key = signing_box.1.get_public_key().await?;

    Ok(ResultOfSigningBoxGetPublicKey {
        pubkey: hex::encode(&key)
    })
}
    
#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ParamsOfSigningBoxSign {
    /// Signing Box handle.
    pub signing_box: SigningBoxHandle,
    /// Unsigned user data. Must be encoded with `base64`.
    pub unsigned: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ResultOfSigningBoxSign {
    /// Data signature. Encoded with `hex`.
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

/// Removes signing box from SDK.
#[api_function]
pub fn remove_signing_box(
    context: Arc<ClientContext>,
    params: RegisteredSigningBox,
) -> ClientResult<()> {
    context.boxes.signing_boxes.remove(&params.handle.0);
    Ok(())
}


#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct EncryptionBoxHandle(pub u32);

impl From<u32> for EncryptionBoxHandle {
    fn from(handle: u32) -> Self {
        Self(handle)
    }
}

/// Encryption box information
#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct EncryptionBoxInfo {
    pub hdpath: Option<String>,
    pub algorithm: Option<String>,
    pub options: Option<Value>,
    pub public: Option<Value>,
}

pub type Base64 = String;
pub type BlobUrl = String;

/// Encryption box data
#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub enum Data {
    Base64(Base64),
    BlobUrl(BlobUrl),
}

impl Default for Data {
    fn default() -> Self {
        Self::Base64(String::new())
    }
}

#[async_trait::async_trait]
pub trait EncryptionBox {
    /// Gets encryption box information
    async fn get_info(&self) -> ClientResult<EncryptionBoxInfo>;
    /// Encrypts data
    async fn encrypt(&self, data: &Data) -> ClientResult<Data>;
    /// Decrypts data
    async fn decrypt(&self, data: &Data) -> ClientResult<Data>;
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct RegisteredEncryptionBox {
    /// Handle of the encryption box
    pub handle: EncryptionBoxHandle,
}

/// Registers an application implemented encryption box.
pub async fn register_encryption_box(
    context: std::sync::Arc<ClientContext>,
    encryption_box: impl EncryptionBox + Send + Sync + 'static,
) -> ClientResult<RegisteredEncryptionBox> {
    let id = context.get_next_id();
    context.boxes.encryption_boxes.insert(id, Box::new(encryption_box));

    Ok(RegisteredEncryptionBox {
        handle: EncryptionBoxHandle(id),
    })
}

fn get_registered_encryption_box<'context>(
    context: &'context Arc<ClientContext>,
    handle: &EncryptionBoxHandle
) -> ClientResult<ReadGuard<'context, u32, Box<dyn EncryptionBox + Send + Sync>>> {
    context.boxes.encryption_boxes
        .get(&handle.0)
        .ok_or(Error::encryption_box_not_registered(handle.0))
}

/// Removes encryption box from SDK
#[api_function]
pub fn remove_encryption_box(
    context: Arc<ClientContext>,
    params: RegisteredEncryptionBox,
) -> ClientResult<()> {
    context.boxes.encryption_boxes.remove(&params.handle.0);
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ParamsOfEncryptionBoxGetInfo {
    /// Encryption box handle
    pub encryption_box: EncryptionBoxHandle,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ResultOfEncryptionBoxGetInfo {
    /// Encryption box information
    pub info: EncryptionBoxInfo,
}

/// Queries info from the given encryption box
#[api_function]
pub async fn encryption_box_get_info(
    context: Arc<ClientContext>,
    params: ParamsOfEncryptionBoxGetInfo,
) -> ClientResult<ResultOfEncryptionBoxGetInfo> {
    Ok(ResultOfEncryptionBoxGetInfo {
        info: get_registered_encryption_box(&context, &params.encryption_box)?
            .val()
            .get_info()
            .await?
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ParamsOfEncryptionBoxEncrypt {
    /// Encryption box handle
    pub encryption_box: EncryptionBoxHandle,
    /// Data to be encrypted
    pub data: Data,
}


#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ResultOfEncryptionBoxEncrypt {
    /// Encrypted data
    pub data: Data,
}

/// Encrypts data using given encryption box
#[api_function]
pub async fn encryption_box_encrypt(
    context: Arc<ClientContext>,
    params: ParamsOfEncryptionBoxEncrypt,
) -> ClientResult<ResultOfEncryptionBoxEncrypt> {
    Ok(ResultOfEncryptionBoxEncrypt {
        data: get_registered_encryption_box(&context, &params.encryption_box)?
            .val()
            .encrypt(&params.data)
            .await?
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ParamsOfEncryptionBoxDecrypt {
    /// Encryption box handle
    pub encryption_box: EncryptionBoxHandle,
    /// Data to be decrypted
    pub data: Data,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ResultOfEncryptionBoxDecrypt {
    /// Decrypted data
    pub data: Data,
}

/// Decrypts data using given encryption box
#[api_function]
pub async fn encryption_box_decrypt(
    context: Arc<ClientContext>,
    params: ParamsOfEncryptionBoxDecrypt,
) -> ClientResult<ResultOfEncryptionBoxDecrypt> {
    Ok(ResultOfEncryptionBoxDecrypt {
        data: get_registered_encryption_box(&context, &params.encryption_box)?
            .val()
            .decrypt(&params.data)
            .await?
    })
}
