/*
 * Copyright 2018-2020 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 *
 */

use crate::client::{AppObject, ClientContext, Error};
use crate::error::ClientResult;
use crate::crypto::{RegisteredSigningBox, SigningBox};
use crate::crypto::boxes::{EncryptionBoxData, EncryptionBoxInfo, EncryptionBox, RegisteredEncryptionBox};

/// Signing box callbacks.
#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
#[serde(tag="type")]
pub enum ParamsOfAppSigningBox {
    /// Get signing box public key
    GetPublicKey,
    /// Sign data
    Sign {
        /// Data to sign encoded as base64
        unsigned: String,
    },
}

/// Returning values from signing box callbacks.
#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
#[serde(tag="type")]
pub enum ResultOfAppSigningBox {
    /// Result of getting public key
    GetPublicKey {
        /// Signing box public key
        public_key: String,
    },
    /// Result of signing data
    Sign {
        /// Data signature encoded as hex
        signature: String,
    },
}

struct ExternalSigningBox {
    app_object: AppObject<ParamsOfAppSigningBox, ResultOfAppSigningBox>,
}

impl ExternalSigningBox {
    pub fn new(app_object: AppObject<ParamsOfAppSigningBox, ResultOfAppSigningBox>) -> Self {
        Self { app_object }
    }
}

#[async_trait::async_trait]
impl SigningBox for ExternalSigningBox {
    async fn get_public_key(&self) -> ClientResult<Vec<u8>> {
        let response = self.app_object.call(ParamsOfAppSigningBox::GetPublicKey).await?;

        match response {
            ResultOfAppSigningBox::GetPublicKey { public_key } => {
               crate::encoding::hex_decode(&public_key)
            },
            _ => Err(Error::unexpected_callback_response(
                "SigningBoxGetPublicKey", &response))
        }
    }

    async fn sign(&self, unsigned: &[u8]) -> ClientResult<Vec<u8>> {
        let response = self.app_object.call(ParamsOfAppSigningBox::Sign { 
            unsigned: base64::encode(unsigned)
        }).await?;

        match response {
            ResultOfAppSigningBox::Sign { signature: signed } => {
               crate::encoding::hex_decode(&signed)
            },
            _ => Err(Error::unexpected_callback_response(
                "SigningBoxSign", &response))
        }
    }
}

/// Register an application implemented signing box.
#[api_function]
pub(crate) async fn register_signing_box(
    context: std::sync::Arc<ClientContext>,
    app_object: AppObject<ParamsOfAppSigningBox, ResultOfAppSigningBox>,
) -> ClientResult<RegisteredSigningBox> {
    crate::crypto::register_signing_box(context, ExternalSigningBox::new(app_object)).await
}

/// Encryption box callbacks.
#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
#[serde(tag="type")]
pub enum ParamsOfAppEncryptionBox {
    /// Get encryption box info
    GetInfo,
    /// Encrypt data
    Encrypt {
        data: EncryptionBoxData,
    },
    /// Decrypt data
    Decrypt {
        data: EncryptionBoxData,
    }
}

/// Returning values from signing box callbacks.
#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
#[serde(tag="type")]
pub enum ResultOfAppEncryptionBox {
    /// Result of getting encryption box info
    GetInfo {
        info: EncryptionBoxInfo,
    },
    /// Result of encrypting data
    Encrypt {
        /// Encrypted data enumeration
        data: EncryptionBoxData,
    },
    /// Result of decrypting data
    Decrypt {
        /// Decrypted data enumeration
        data: EncryptionBoxData,
    },
}

struct ExternalEncryptionBox {
    app_object: AppObject<ParamsOfAppEncryptionBox, ResultOfAppEncryptionBox>,
}

impl ExternalEncryptionBox {
    pub fn new(app_object: AppObject<ParamsOfAppEncryptionBox, ResultOfAppEncryptionBox>) -> Self {
        Self { app_object }
    }
}

#[async_trait::async_trait]
impl EncryptionBox for ExternalEncryptionBox {
    async fn get_info(&self) -> ClientResult<EncryptionBoxInfo> {
        let response = self.app_object.call(ParamsOfAppEncryptionBox::GetInfo).await?;

        match response {
            ResultOfAppEncryptionBox::GetInfo { info } => Ok(info),
            _ => Err(Error::unexpected_callback_response(
                "EncryptionBoxGetInfo", &response))
        }
    }

    async fn encrypt(&self, data: &EncryptionBoxData) -> ClientResult<EncryptionBoxData> {
        let response =
            self.app_object.call(ParamsOfAppEncryptionBox::Encrypt { data: data.clone() }).await?;

        match response {
            ResultOfAppEncryptionBox::Encrypt { data } => Ok(data),
            _ => Err(Error::unexpected_callback_response(
                "EncryptionBoxEncrypt", &response))
        }
    }

    async fn decrypt(&self, data: &EncryptionBoxData) -> ClientResult<EncryptionBoxData> {
        let response =
            self.app_object.call(ParamsOfAppEncryptionBox::Decrypt { data: data.clone() }).await?;

        match response {
            ResultOfAppEncryptionBox::Decrypt { data } => Ok(data),
            _ => Err(Error::unexpected_callback_response(
                "EncryptionBoxDecrypt", &response))
        }
    }
}

/// Register an application implemented encryption box.
#[api_function]
pub(crate) async fn register_encryption_box(
    context: std::sync::Arc<ClientContext>,
    app_object: AppObject<ParamsOfAppEncryptionBox, ResultOfAppEncryptionBox>,
) -> ClientResult<RegisteredEncryptionBox> {
    crate::crypto::register_encryption_box(context, ExternalEncryptionBox::new(app_object)).await
}
