/*
* Copyright 2018-2021 EverX Labs Ltd.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific EVERX DEV software governing permissions and
* limitations under the License.
*/

use std::sync::Arc;

use lockfree::map::ReadGuard;
use serde_json::Value;

use crate::client::ClientContext;
use crate::crypto::{CryptoBoxHandle, Error};
use crate::error::ClientResult;

pub(crate) mod aes;
pub(crate) mod chacha20;
pub(crate) mod nacl_box;
pub(crate) mod nacl_secret_box;

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct EncryptionBoxHandle(pub u32);

impl From<u32> for EncryptionBoxHandle {
    fn from(handle: u32) -> Self {
        Self(handle)
    }
}

/// Encryption box information.
#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct EncryptionBoxInfo {
    /// Derivation path, for instance "m/44'/396'/0'/0/0"
    pub hdpath: Option<String>,
    /// Cryptographic algorithm, used by this encryption box
    pub algorithm: Option<String>,
    /// Options, depends on algorithm and specific encryption box implementation
    pub options: Option<Value>,
    /// Public information, depends on algorithm
    pub public: Option<Value>,
}

#[async_trait::async_trait]
pub trait EncryptionBox: Send + Sync {
    /// Gets encryption box information
    async fn get_info(&self, context: Arc<ClientContext>) -> ClientResult<EncryptionBoxInfo>;
    /// Encrypts data
    async fn encrypt(&self, context: Arc<ClientContext>, data: &String) -> ClientResult<String>;
    /// Decrypts data
    async fn decrypt(&self, context: Arc<ClientContext>, data: &String) -> ClientResult<String>;
    /// Zeroize all secret data
    async fn drop_secret(&self, _crypto_box_handle: CryptoBoxHandle) {
        // Not implemented by default, but must be implemented for encryption boxes that created
        // from crypto boxes.
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct RegisteredEncryptionBox {
    /// Handle of the encryption box.
    pub handle: EncryptionBoxHandle,
}

/// Registers an application implemented encryption box.
pub async fn register_encryption_box(
    context: Arc<ClientContext>,
    encryption_box: impl EncryptionBox + 'static,
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
) -> ClientResult<ReadGuard<'context, u32, Box<dyn EncryptionBox>>> {
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
            .get_info(Arc::clone(&context))
            .await?
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ParamsOfEncryptionBoxEncrypt {
    /// Encryption box handle
    pub encryption_box: EncryptionBoxHandle,
    /// Data to be encrypted, encoded in Base64
    pub data: String,
}


#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ResultOfEncryptionBoxEncrypt {
    /// Encrypted data, encoded in Base64. Padded to cipher block size
    pub data: String,
}

/// Encrypts data using given encryption box
/// Note. Block cipher algorithms pad data to cipher block size so encrypted data can be longer then
/// original data. Client should store the original data size after encryption and use it after
/// decryption to retrieve the original data from decrypted data.
#[api_function]
pub async fn encryption_box_encrypt(
    context: Arc<ClientContext>,
    params: ParamsOfEncryptionBoxEncrypt,
) -> ClientResult<ResultOfEncryptionBoxEncrypt> {
    Ok(ResultOfEncryptionBoxEncrypt {
        data: get_registered_encryption_box(&context, &params.encryption_box)?
            .val()
            .encrypt(Arc::clone(&context), &params.data)
            .await?
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ParamsOfEncryptionBoxDecrypt {
    /// Encryption box handle
    pub encryption_box: EncryptionBoxHandle,
    /// Data to be decrypted, encoded in Base64
    pub data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct ResultOfEncryptionBoxDecrypt {
    /// Decrypted data, encoded in Base64.
    pub data: String,
}

/// Decrypts data using given encryption box
/// Note. Block cipher algorithms pad data to cipher block size so encrypted data can be longer then
/// original data. Client should store the original data size after encryption and use it after
/// decryption to retrieve the original data from decrypted data.
#[api_function]
pub async fn encryption_box_decrypt(
    context: Arc<ClientContext>,
    params: ParamsOfEncryptionBoxDecrypt,
) -> ClientResult<ResultOfEncryptionBoxDecrypt> {
    Ok(ResultOfEncryptionBoxDecrypt {
        data: get_registered_encryption_box(&context, &params.encryption_box)?
            .val()
            .decrypt(Arc::clone(&context), &params.data)
            .await?
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub enum CipherMode {
    CBC,
    CFB,
    CTR,
    ECB,
    OFB,
}

impl Default for CipherMode {
    fn default() -> Self {
        CipherMode::CBC
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum EncryptionAlgorithm {
    AES(aes::AesParamsEB),
    ChaCha20(chacha20::ChaCha20ParamsEB),
    NaclBox(nacl_box::NaclBoxParamsEB),
    NaclSecretBox(nacl_secret_box::NaclSecretBoxParamsEB),
}

impl Default for EncryptionAlgorithm {
    fn default() -> Self {
        EncryptionAlgorithm::AES(Default::default())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct ParamsOfCreateEncryptionBox {
    /// Encryption algorithm specifier including cipher parameters (key, IV, etc)
    pub algorithm: EncryptionAlgorithm,
}

/// Creates encryption box with specified algorithm
#[api_function]
pub async fn create_encryption_box(
    context: Arc<ClientContext>,
    params: ParamsOfCreateEncryptionBox,
) -> ClientResult<RegisteredEncryptionBox> {
    match params.algorithm {
        EncryptionAlgorithm::AES(params) =>
            register_encryption_box(context, aes::AesEncryptionBox::new(params)?).await,

        EncryptionAlgorithm::ChaCha20(params) =>
            register_encryption_box(context, chacha20::ChaCha20EncryptionBox::new(params, None)?).await,

        EncryptionAlgorithm::NaclBox(params) =>
            register_encryption_box(context, nacl_box::NaclEncryptionBox::new(params, None)).await,

        EncryptionAlgorithm::NaclSecretBox(params) =>
            register_encryption_box(context, nacl_secret_box::NaclSecretEncryptionBox::new(params, None)).await,
    }
}
