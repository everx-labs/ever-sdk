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

use sha2::{Digest};
use crate::client::ClientContext;
use crate::types::{ApiResult, InputData, OutputEncoding};
use crate::serialization::{default_output_encoding_base64};

//--------------------------------------------------------------------------------------------- sha

#[derive(Deserialize)]
pub struct ParamsOfHash {
    /// Input data for hash calculation.
    pub data: InputData,
    /// Encoding of calculated hash.
    #[serde(default = "default_output_encoding_hex")]
    pub output_encoding: OutputEncoding,
}

#[derive(Serialize)]
pub struct ResultOfHash {
    /// Calculated hash of input `data` encoded according to `output_encoding`.
    pub hash: String,
}

/// Calculates SHA256 hash of specified data.
pub fn sha256(
    _context: &mut ClientContext,
    params: ParamsOfHash,
) -> ApiResult<ResultOfHash> {
    let mut hasher = sha2::Sha256::new();
    hasher.input(bytes);
    Ok(ResultOfHash {
        hash: params.output_encoding.encode(hasher.result().to_vec())?
    })
}


/// Calculates SHA512 hash of specified data.
pub fn sha512(
    _context: &mut ClientContext,
    params: ParamsOfHash,
) -> ApiResult<ResultOfHash> {
    let mut hasher = sha2::Sha512::new();
    hasher.input(bytes);
    Ok(ResultOfHash {
        hash: params.output_encoding.encode(hasher.result().to_vec())?
    })
}
