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
pub(crate) struct ParamsOfChaCha20 {
    /// Source data that must be encrypted/decrypted. Must be encoded with `base64`.
    pub data: String,
    /// 256-bit key. Must be encoded with `hex`.
    key: String,
    /// 96-bit nonce. Must be encoded with `hex`.
    nonce: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfChaCha20 {
    /// Encrypted/decrypted data. Encoded with `base64`.
    pub data: String,
}

pub(crate) fn cha_cha_20(
    _context: &mut ClientContext,
    params: ParamsOfChaCha20,
) -> ApiResult<ResultOfChaCha20> {
    let key = hex_decode(&params.key)?;
    let nonce = hex_decode(&params.nonce)?;
    let mut cipher = chacha20::ChaCha20::new(Key::from_slice(&key), Nonce::from_slice(&nonce));
    let mut data = base64_decode(&params.data)?;
    cipher.apply_keystream(&mut data);
    Ok(ResultOfChaCha20 {
        data: base64::encode(&data),
    })
}
