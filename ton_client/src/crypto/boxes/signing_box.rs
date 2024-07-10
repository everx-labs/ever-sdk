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

use crate::client::ClientContext;
use crate::crypto::{CryptoBoxHandle, Error};
use crate::crypto::KeyPair;
use crate::error::ClientResult;

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct SigningBoxHandle(pub u32);

impl From<u32> for SigningBoxHandle {
    fn from(handle: u32) -> Self {
        Self(handle)
    }
}

#[async_trait::async_trait]
pub trait SigningBox: Send + Sync {
    /// Get public key of key pair
    async fn get_public_key(&self, context: Arc<ClientContext>) -> ClientResult<Vec<u8>>;
    /// Sign data with key pair
    async fn sign(&self, context: Arc<ClientContext>, unsigned: &[u8]) -> ClientResult<Vec<u8>>;
    /// Zeroize all secret data
    async fn drop_secret(&self, _crypto_box_handle: CryptoBoxHandle) {
        // Not implemented by default, but must be implemented for signing boxes that created from
        // crypto boxes.
    }
}

pub(crate) struct KeysSigningBox {
    sign_key: ed25519_dalek::SigningKey
}

impl KeysSigningBox {
    pub fn new(sign_key: ed25519_dalek::SigningKey) -> Self {
        Self {
            sign_key
        }
    }

    pub fn from_encoded(key_pair: KeyPair) -> ClientResult<Self> {
        key_pair.decode().map(|pair| Self::new(pair))
    }
}

#[async_trait::async_trait]
impl SigningBox for KeysSigningBox {
    async fn get_public_key(&self, _context: Arc<ClientContext>) -> ClientResult<Vec<u8>> {
        Ok(self.sign_key.verifying_key().to_bytes().to_vec())
    }

    async fn sign(&self, _context: Arc<ClientContext>, unsigned: &[u8]) -> ClientResult<Vec<u8>> {
        crate::crypto::internal::sign_using_keys(unsigned, &self.sign_key).map(|result| result.1)
    }
}

/// Creates a default signing box implementation.
#[api_function]
pub async fn get_signing_box(
    context: Arc<ClientContext>,
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
    context: Arc<ClientContext>,
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

    let key = signing_box.1.get_public_key(Arc::clone(&context)).await?;

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

    let signed = signing_box.1.sign(Arc::clone(&context), &unsigned).await?;

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
