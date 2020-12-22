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

use crate::boc::internal::deserialize_object_from_base64;
use crate::boc::Error;
use crate::client::ClientContext;
use crate::error::ClientResult;

#[derive(Serialize, Deserialize, Clone, ApiType)]
pub struct ParamsOfGetCodeFromTvc {
    /// Contract TVC image encoded as base64
    pub tvc: String,
}

#[derive(Serialize, Deserialize, Clone, ApiType)]
pub struct ResultOfGetCodeFromTvc {
    /// Contract code encoded as base64
    pub code: String,
}

/// Extracts code from TVC contract image
#[api_function]
pub fn get_code_from_tvc(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfGetCodeFromTvc,
) -> ClientResult<ResultOfGetCodeFromTvc> {
    let object = deserialize_object_from_base64::<ton_block::StateInit>(&params.tvc, "TVC")?;
    
    let code = object.object.code.ok_or(Error::invalid_boc("TVC image has no code"))?;

    Ok(ResultOfGetCodeFromTvc {
        code: super::internal::serialize_cell_to_base64(&code, "code")?,
    })
}
