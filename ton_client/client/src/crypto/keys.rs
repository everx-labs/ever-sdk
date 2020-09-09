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
*/

use crate::error::{ApiResult};
use crate::encoding::{hex_decode};
use base64::URL_SAFE;
use crate::client::ClientContext;
use crate::crypto::internal;
use ed25519_dalek::Keypair;
use crate::crypto::internal::ton_crc16;

//----------------------------------------------------------------------------------------- KeyPair

#[derive(Serialize, Deserialize, Clone, TypeInfo)]
pub struct KeyPair {
    pub public: String,
    pub secret: String,
}

impl KeyPair {
    pub fn new(public: String, secret: String) -> KeyPair {
        KeyPair { public, secret }
    }

    pub fn decode(&self) -> ApiResult<Keypair> {
        Ok(Keypair {
            public: internal::decode_public_key(&self.public)?,
            secret: internal::decode_secret_key(&self.secret)?,
        })
    }
}

//----------------------------------------------------------- convert_public_key_to_ton_safe_format

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfConvertPublicKeyToTonSafeFormat {
    /// Public key.
    pub public_key: String,
}

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfConvertPublicKeyToTonSafeFormat {
    /// Public key represented in TON safe format.
    pub ton_public_key: String,
}

/// Converts public key to ton safe_format.
pub fn convert_public_key_to_ton_safe_format(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfConvertPublicKeyToTonSafeFormat,
) -> ApiResult<ResultOfConvertPublicKeyToTonSafeFormat> {
    let public_key = hex_decode(&params.public_key)?;
    let mut ton_public_key: Vec<u8> = Vec::new();
    ton_public_key.push(0x3e);
    ton_public_key.push(0xe6);
    ton_public_key.extend_from_slice(&public_key);
    let hash = ton_crc16(&ton_public_key);
    ton_public_key.push((hash >> 8) as u8);
    ton_public_key.push((hash & 255) as u8);
    Ok(ResultOfConvertPublicKeyToTonSafeFormat {
        ton_public_key: base64::encode_config(&ton_public_key, URL_SAFE)
    })
}

//----------------------------------------------------------------------- generate_random_sign_keys

/// Generates random ed25519 key pair.
pub fn generate_random_sign_keys(_context: std::sync::Arc<ClientContext>) -> ApiResult<KeyPair> {
    let mut rng = rand::thread_rng();
    let keypair = ed25519_dalek::Keypair::generate(&mut rng);
    Ok(KeyPair::new(
        hex::encode(keypair.public.to_bytes()),
        hex::encode(keypair.secret.to_bytes()),
    ))
}
