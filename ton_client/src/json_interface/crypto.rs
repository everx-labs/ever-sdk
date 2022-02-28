/*
 * Copyright 2018-2021 TON Labs LTD.
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

use std::sync::Arc;

use crate::client::{AppObject, ClientContext, Error};
use crate::crypto::{EncryptionBoxInfo, RegisteredEncryptionBox, RegisteredSigningBox, SigningBox};
use crate::crypto::boxes::crypto_box::{AppPasswordProvider, ParamsOfCreateCryptoBox, RegisteredCryptoBox, ResultOfGetPassword};
use crate::crypto::boxes::encryption_box::EncryptionBox;
use crate::crypto::internal::key256;
use crate::encoding::{base64_decode, hex_decode};
use crate::error::ClientResult;

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
    async fn get_public_key(&self, _context: Arc<ClientContext>) -> ClientResult<Vec<u8>> {
        let response = self.app_object.call(ParamsOfAppSigningBox::GetPublicKey).await?;

        match response {
            ResultOfAppSigningBox::GetPublicKey { public_key } => {
               crate::encoding::hex_decode(&public_key)
            },
            _ => Err(Error::unexpected_callback_response(
                "SigningBoxGetPublicKey", &response))
        }
    }

    async fn sign(&self, _context: Arc<ClientContext>, unsigned: &[u8]) -> ClientResult<Vec<u8>> {
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
        /// Data, encoded in Base64
        data: String,
    },
    /// Decrypt data
    Decrypt {
        /// Data, encoded in Base64
        data: String,
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
        /// Encrypted data, encoded in Base64
        data: String,
    },
    /// Result of decrypting data
    Decrypt {
        /// Decrypted data, encoded in Base64
        data: String,
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
    async fn get_info(&self, context: Arc<ClientContext>) -> ClientResult<EncryptionBoxInfo> {
        let response = self.app_object.call(ParamsOfAppEncryptionBox::GetInfo).await?;

        match response {
            ResultOfAppEncryptionBox::GetInfo { info } => Ok(info),
            _ => Err(Error::unexpected_callback_response(
                "EncryptionBoxGetInfo", &response))
        }
    }

    async fn encrypt(&self, context: Arc<ClientContext>, data: &String) -> ClientResult<String> {
        let response =
            self.app_object.call(ParamsOfAppEncryptionBox::Encrypt { data: data.clone() }).await?;

        match response {
            ResultOfAppEncryptionBox::Encrypt { data } => Ok(data),
            _ => Err(Error::unexpected_callback_response(
                "EncryptionBoxEncrypt", &response))
        }
    }

    async fn decrypt(&self, context: Arc<ClientContext>, data: &String) -> ClientResult<String> {
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

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
#[serde(tag="type")]
pub enum ParamsOfAppPasswordProvider {
    GetPassword {
        encryption_public_key: String,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
#[serde(tag="type")]
pub enum ResultOfAppPasswordProvider {
    GetPassword {
        /// User's password hash, encrypted and encoded to base64.
        /// Crypto box uses this password to decrypt its secret (seed phrase).
        /// Password is encrypted with `encryption_public_key`.
        encrypted_password: String,
        /// Hex encoded public key of the key pair, used for password encryption in client
        /// application.
        app_encryption_pubkey: String,
    }
}

struct ExternalPasswordProvider {
    app_object: AppObject<ParamsOfAppPasswordProvider, ResultOfAppPasswordProvider>,
}

#[async_trait::async_trait]
impl AppPasswordProvider for ExternalPasswordProvider {
    async fn get_password(&self, encryption_public_key: &sodalite::BoxPublicKey) -> ClientResult<ResultOfGetPassword> {
        let ResultOfAppPasswordProvider::GetPassword { encrypted_password, app_encryption_pubkey } =
            self.app_object.call(
                ParamsOfAppPasswordProvider::GetPassword {
                    encryption_public_key: hex::encode(encryption_public_key),
                },
            ).await?;

        Ok(ResultOfGetPassword {
            encrypted_password: base64_decode(&encrypted_password)?,
            app_encryption_pubkey: key256(&hex_decode(&app_encryption_pubkey)?)?,
        })
    }
}

/// Creates Crypto Box.
///
/// Crypto Box is a root crypto object, that encapsulates some secret (seed phrase usually)
/// in encrypted form and acts as a factory for all crypto primitives used in SDK:
/// keys for signing and encryption, derived from this secret.
///
/// Crypto Box encrypts original Seed Phrase with salt and some secret that is retrieved
/// in runtime via `password_provider` callback, implemented on Application side.
///
/// When used, decrypted secret has shown up in core library's memory for a very short period
/// of time and then is immediately overwritten with zeroes.
#[api_function]
pub(crate) async fn create_crypto_box(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfCreateCryptoBox,
    password_provider: AppObject<ParamsOfAppPasswordProvider, ResultOfAppPasswordProvider>,
) -> ClientResult<RegisteredCryptoBox> {
    crate::crypto::boxes::crypto_box::create_crypto_box(
        context,
        params,
        Arc::new(ExternalPasswordProvider { app_object: password_provider }),
    ).await
}
