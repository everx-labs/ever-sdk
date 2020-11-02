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

extern crate scrypt;
use crate::crypto;
use crate::error::{ClientResult};
use crate::client::ClientContext;
use crate::encoding::base64_decode;

//------------------------------------------------------------------------------------------ scrypt

#[derive(Serialize, Deserialize, ApiType)]
pub struct ParamsOfScrypt {
    /// The password bytes to be hashed.
    /// Must be encoded with `base64`.
    pub password: String,
    /// A salt bytes that modifies the hash to protect against Rainbow table attacks.
    /// Must be encoded with `base64`.
    pub salt: String,
    /// CPU/memory cost parameter
    pub log_n: u8,
    /// The block size parameter, which fine-tunes sequential memory read size and performance.
    pub r: u32,
    /// Parallelization parameter.
    pub p: u32,
    /// Intended output length in octets of the derived key.
    pub dk_len: usize,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfScrypt {
    /// Derived key. Encoded with `hex`.
    pub key: String,
}

#[doc(summary = "Perform `scrypt` encryption")]
/// Derives key from `password` and `key` using `scrypt` algorithm.
/// See [https://en.wikipedia.org/wiki/Scrypt].
///
/// # Arguments
/// - `log_n` - The log2 of the Scrypt parameter `N`
/// - `r` - The Scrypt parameter `r`
/// - `p` - The Scrypt parameter `p`
/// # Conditions
/// - `log_n` must be less than `64`
/// - `r` must be greater than `0` and less than or equal to `4294967295`
/// - `p` must be greater than `0` and less than `4294967295`
/// # Recommended values sufficient for most use-cases
/// - `log_n = 15` (`n = 32768`)
/// - `r = 8`
/// - `p = 1`
#[api_function]
pub fn scrypt(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfScrypt,
) -> ClientResult<ResultOfScrypt> {
    let mut key = Vec::new();
    key.resize(params.dk_len, 0);
    let scrypt_params = scrypt::ScryptParams::new(params.log_n, params.r, params.p)
        .map_err(|err| crypto::Error::scrypt_failed(err))?;
    let password = base64_decode(&params.password)?;
    let salt = base64_decode(&params.salt)?;
    scrypt::scrypt(&password, &salt, &scrypt_params, &mut key)
        .map_err(|err| crypto::Error::scrypt_failed(err))?;
    Ok(ResultOfScrypt {
        key: hex::encode(&key)
    })
}

