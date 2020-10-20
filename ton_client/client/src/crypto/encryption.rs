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

use crate::client::ClientContext;
use crate::types::{base64_decode, hex_decode, ApiResult};
use chacha20;
use chacha20::cipher::{NewStreamCipher, SyncStreamCipher};
use chacha20::{Key, Nonce};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum Cipher {
    ChaCha20 {
        /// 256-bit key. Must be encoded with `hex`.
        key: String,
        /// 96-bit nonce. Must be encoded with `hex`.
        nonce: String,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum Decipher {
    ChaCha20 {
        /// 256-bit key. Must be encoded with `hex`.
        key: String,
        /// 96-bit nonce. Must be encoded with `hex`.
        nonce: String,
    },
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ParamsOfEncrypt {
    /// Decrypted data. Must be encoded with `base64`.
    pub data: String,
    /// Cipher used.
    pub cipher: Cipher,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfEncrypt {
    /// Encrypted data. Encoded with `base64`.
    pub data: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ParamsOfDecrypt {
    /// Decrypted data. Must be encoded with `base64`.
    pub data: String,
    /// Decipher used.
    pub decipher: Decipher,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfDecrypt {
    /// Decrypted data. Encoded with `base64`.
    pub data: String,
}

fn cha_cha_20(data: String, key: String, nonce: String) -> ApiResult<String> {
    let key = hex_decode(&key)?;
    let nonce = hex_decode(&nonce)?;
    let mut cipher = chacha20::ChaCha20::new(Key::from_slice(&key), Nonce::from_slice(&nonce));
    let mut data = base64_decode(&data)?;
    cipher.apply_keystream(&mut data);
    Ok(base64::encode(&data))
}

pub(crate) fn encrypt(
    _context: &mut ClientContext,
    params: ParamsOfEncrypt,
) -> ApiResult<ResultOfEncrypt> {
    match params.cipher {
        Cipher::ChaCha20 { key, nonce } => Ok(ResultOfEncrypt {
            data: cha_cha_20(params.data, key, nonce)?,
        }),
    }
}

pub(crate) fn decrypt(
    _context: &mut ClientContext,
    params: ParamsOfDecrypt,
) -> ApiResult<ResultOfDecrypt> {
    match params.decipher {
        Decipher::ChaCha20 { key, nonce } => Ok(ResultOfDecrypt {
            data: cha_cha_20(params.data, key, nonce)?,
        }),
    }
}
