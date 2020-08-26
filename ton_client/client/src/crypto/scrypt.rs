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

use crate::types::{ApiResult, ApiError, InputData, OutputEncoding};
use crate::client::ClientContext;
use crate::serialization::default_output_encoding_base64;

//------------------------------------------------------------------------------------------ scrypt

#[derive(Deserialize)]
pub struct ParamsOfScrypt {
    pub password: InputData,
    pub salt: InputData,
    pub log_n: u8,
    pub r: u32,
    pub p: u32,
    pub dk_len: usize,
    #[serde(default = "default_result_encoding_base64")]
    pub output_encoding: OutputEncoding,
}

#[derive(Serialize)]
pub struct ResultOfScrypt {
    bytes: String,
}

/// Perform `scrypt` encryption.
/// See [https://en.wikipedia.org/wiki/Scrypt].
pub fn scrypt(
    _context: &mut ClientContext,
    params: ParamsOfScrypt,
) -> ApiResult<ResultOfScrypt> {
    let mut result = Vec::new();
    result.resize(dk_len, 0);
    let scrypt_params = scrypt::ScryptParams::new(params.log_n, params.r, params.p)
        .map_err(|err| ApiError::crypto_scrypt_failed(err))?;
    scrypt::scrypt(
        &(params.password.decode()?),
        &(params.salt.decode()?),
        &scrypt_params,
        &mut result,
    ).map_err(|err| ApiError::crypto_scrypt_failed(err))?;
    Ok(ResultOfScrypt {
        bytes: scrypt_params.output_encoding.encode(bytes)?
    })
}

