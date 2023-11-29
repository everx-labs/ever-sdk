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
*/

use crate::client::ClientContext;
use crate::crypto;
use crate::crypto::internal::{decode_public_key, decode_secret_key, sign_using_keys, ton_crc16};
use crate::encoding::{base64_decode, hex_decode};
use crate::error::ClientResult;
use base64::URL_SAFE;
use ed25519_dalek::SigningKey;
use std::fmt::{Debug, Formatter};

use super::internal::{hex_decode_secret_const, SecretBufConst};

pub(crate) fn strip_secret(secret: &str) -> String {
    const SECRET_SHOW_LEN: usize = 8;
    if secret.len() <= SECRET_SHOW_LEN {
        return format!(r#""{}""#, secret);
    }

    format!(
        r#""{}..." ({} chars)"#,
        &secret[..SECRET_SHOW_LEN],
        secret.len(),
    )
}

//----------------------------------------------------------------------------------------- KeyPair
///
#[derive(Serialize, Deserialize, Clone, ApiType, Default, PartialEq, zeroize::ZeroizeOnDrop)]
pub struct KeyPair {
    /// Public key - 64 symbols hex string
    pub public: String,
    /// Private key - u64 symbols hex string
    pub secret: String,
}

impl KeyPair {
    pub fn new(public: String, secret: String) -> KeyPair {
        KeyPair { public, secret }
    }

    pub fn decode(&self) -> ClientResult<SigningKey> {
        let secret = decode_secret_key(&self.secret)?;
        let public = decode_public_key(&self.public)?;

        if secret.verifying_key() != public {
            return Err(super::Error::invalid_public_key(
                "public key doesn't correspond to secret key", &self.public
            ));
        }

        Ok(secret)
    }
}

impl Debug for KeyPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"KeyPair {{ public: "{}", secret: {} }}"#,
            self.public,
            strip_secret(&self.secret)
        )
    }
}

//----------------------------------------------------------- convert_public_key_to_ton_safe_format
///
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfConvertPublicKeyToTonSafeFormat {
    /// Public key - 64 symbols hex string
    pub public_key: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfConvertPublicKeyToTonSafeFormat {
    /// Public key represented in TON safe format.
    pub ton_public_key: String,
}

/// Converts public key to ton safe_format
#[api_function]
pub fn convert_public_key_to_ton_safe_format(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfConvertPublicKeyToTonSafeFormat,
) -> ClientResult<ResultOfConvertPublicKeyToTonSafeFormat> {
    let public_key = hex_decode(&params.public_key)?;
    let mut ton_public_key: Vec<u8> = Vec::new();
    ton_public_key.push(0x3e);
    ton_public_key.push(0xe6);
    ton_public_key.extend_from_slice(&public_key);
    let hash = ton_crc16(&ton_public_key);
    ton_public_key.push((hash >> 8) as u8);
    ton_public_key.push((hash & 255) as u8);
    Ok(ResultOfConvertPublicKeyToTonSafeFormat {
        ton_public_key: base64::encode_config(&ton_public_key, URL_SAFE),
    })
}

//----------------------------------------------------------------------- generate_random_sign_keys

/// Generates random ed25519 key pair.
#[api_function]
pub fn generate_random_sign_keys(_context: std::sync::Arc<ClientContext>) -> ClientResult<KeyPair> {
    let bytes = SecretBufConst(rand::random());
    let sign_key = SigningKey::from_bytes(&bytes.0);
    Ok(KeyPair::new(
        hex::encode(&sign_key.verifying_key().to_bytes()),
        hex::encode(bytes),
    ))
}

//-------------------------------------------------------------------------------------------- sign

///
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfSign {
    /// Data that must be signed encoded in `base64`.
    pub unsigned: String,
    /// Sign keys.
    pub keys: KeyPair,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfSign {
    /// Signed data combined with signature encoded in `base64`.
    pub signed: String,
    /// Signature encoded in `hex`.
    pub signature: String,
}

/// Signs a data using the provided keys.
#[api_function]
pub fn sign(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfSign,
) -> ClientResult<ResultOfSign> {
    let (signed, signature) =
        sign_using_keys(&base64_decode(&params.unsigned)?, &params.keys.decode()?)?;
    Ok(ResultOfSign {
        signed: base64::encode(&signed),
        signature: hex::encode(signature),
    })
}

//-------------------------------------------------------------------------------- verify_signature

///
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfVerifySignature {
    /// Signed data that must be verified encoded in `base64`.
    pub signed: String,
    /// Signer's public key - 64 symbols hex string
    pub public: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfVerifySignature {
    /// Unsigned data encoded in `base64`.
    pub unsigned: String,
}

/// Verifies signed data using the provided public key.
/// Raises error if verification is failed.
#[api_function]
pub fn verify_signature(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfVerifySignature,
) -> ClientResult<ResultOfVerifySignature> {
    let mut unsigned: Vec<u8> = Vec::new();
    let signed = base64_decode(&params.signed)?;
    unsigned.resize(signed.len(), 0);
    let len = sodalite::sign_attached_open(
        &mut unsigned,
        &signed,
        &hex_decode_secret_const(&params.public)?.0,
    )
    .map_err(|_| crypto::Error::nacl_sign_failed("verify signature failed"))?;
    unsigned.resize(len, 0);
    Ok(ResultOfVerifySignature {
        unsigned: base64::encode(&unsigned),
    })
}
