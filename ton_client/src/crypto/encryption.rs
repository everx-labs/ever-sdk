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

use crate::client::ClientContext;
use crate::encoding::{base64_decode, hex_decode};
use crate::error::ClientResult;
use super::Error;
use super::internal::hex_decode_secret;
use chacha20::cipher::{NewStreamCipher, SyncStreamCipher};
use chacha20::{Key, Nonce};
use std::sync::Arc;
use zeroize::ZeroizeOnDrop;

#[derive(Serialize, Deserialize, ApiType, Default, ZeroizeOnDrop)]
pub struct ParamsOfChaCha20 {
    /// Source data to be encrypted or decrypted. Must be encoded with `base64`.
    #[zeroize(skip)]
    pub data: String,
    /// 256-bit key. Must be encoded with `hex`.
    pub key: String,
    /// 96-bit nonce. Must be encoded with `hex`.
    #[zeroize(skip)]
    pub nonce: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfChaCha20 {
    /// Encrypted/decrypted data. Encoded with `base64`.
    pub data: String,
}

/// Performs symmetric `chacha20` encryption.
#[api_function]
pub fn chacha20(
    _context: Arc<ClientContext>,
    params: ParamsOfChaCha20,
) -> ClientResult<ResultOfChaCha20> {
    let key = hex_decode_secret(&params.key)?;
    let nonce = hex_decode(&params.nonce)?;
    if key.len() != 32 {
        return Err(Error::invalid_key_size(key.len(), &[32]));
    }
    if nonce.len() != 12 {
        return Err(Error::invalid_nonce_size(nonce.len(), &[12]));
    }
    let mut cipher = chacha20::ChaCha20::new(Key::from_slice(&key), Nonce::from_slice(&nonce));
    let mut data = base64_decode(&params.data)?;
    cipher.apply_keystream(&mut data);
    Ok(ResultOfChaCha20 {
        data: base64::encode(&data),
    })
}
