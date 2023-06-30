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

use crate::boc::internal::deserialize_cell_from_boc;
use crate::client::ClientContext;
use crate::error::ClientResult;

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ParamsOfGetBocHash {
    /// BOC encoded as base64 or BOC handle
    pub boc: String,
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ResultOfGetBocHash {
    /// BOC root hash encoded with hex
    pub hash: String,
}

/// Calculates BOC root hash
#[api_function]
pub fn get_boc_hash(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfGetBocHash,
) -> ClientResult<ResultOfGetBocHash> {
    let (_, cell) = deserialize_cell_from_boc(&context, &params.boc, "")?;
    let hash = cell.repr_hash().as_hex_string();
    Ok(ResultOfGetBocHash { hash })
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ParamsOfGetBocDepth {
    /// BOC encoded as base64 or BOC handle
    pub boc: String,
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ResultOfGetBocDepth {
    /// BOC root cell depth
    pub depth: u32,
}

/// Calculates BOC depth
#[api_function]
pub fn get_boc_depth(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfGetBocDepth,
) -> ClientResult<ResultOfGetBocDepth> {
    let (_, cell) = deserialize_cell_from_boc(&context, &params.boc, "")?;
    Ok(ResultOfGetBocDepth {
        depth: cell.repr_depth() as u32
    })
}
