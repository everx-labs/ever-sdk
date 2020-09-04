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

use crate::error::{ApiResult, ApiError};
use crate::client::ClientContext;
use crate::encoding::base64_decode;

//------------------------------------------------------------------------------------------ scrypt

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfScrypt {
    /// The string of characters to be hashed
    pub password: String,
    /// A string of characters that modifies the hash to protect against Rainbow table attacks
    pub salt: String,
    /// CPU/memory cost parameter
    pub log_n: u8,
    /// The blocksize parameter, which fine-tunes sequential memory read size and performance. 8 is commonly used
    pub r: u32,
    /// Parallelization parameter
    pub p: u32,
    /// Intended output length in octets of the derived key
    pub dk_len: usize,
}

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfScrypt {
    /// Scrypt result hash. Encoded with `hex`.
    pub bytes: String,
}

#[doc(summary = "Perform `scrypt` encryption")]
/// Perform `scrypt` encryption.
/// See [https://en.wikipedia.org/wiki/Scrypt].
pub fn scrypt(
    _context: &mut ClientContext,
    params: ParamsOfScrypt,
) -> ApiResult<ResultOfScrypt> {
    let mut result = Vec::new();
    result.resize(params.dk_len, 0);
    let scrypt_params = scrypt::ScryptParams::new(params.log_n, params.r, params.p)
        .map_err(|err| ApiError::crypto_scrypt_failed(err))?;
    scrypt::scrypt(
        &(base64_decode(&params.password)?),
        &(base64_decode(&params.salt)?),
        &scrypt_params,
        &mut result,
    ).map_err(|err| ApiError::crypto_scrypt_failed(err))?;
    Ok(ResultOfScrypt {
        bytes: base64::encode(&result)
    })
}

