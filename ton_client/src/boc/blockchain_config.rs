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

use crate::boc::Error;
use crate::client::ClientContext;
use crate::error::ClientResult;
use ton_block::Serializable;
use super::internal::deserialize_object_from_boc;

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ParamsOfGetBlockchainConfig {
    /// Key block BOC encoded as base64
    pub block_boc: String,
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ResultOfGetBlockchainConfig {
    /// Blockchain config BOC encoded as base64
    pub config_boc: String,
}

#[api_function]
pub async fn get_blockchain_config(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfGetBlockchainConfig,
) -> ClientResult<ResultOfGetBlockchainConfig> {
    let object = deserialize_object_from_boc::<ton_block::Block>(&context, &params.block_boc, "block").await?;

    let extra = object
        .object
        .read_extra()
        .map_err(|err| Error::invalid_boc(format!("can not read `extra` from block: {}", err)))?;

    let master = extra
        .read_custom()
        .map_err(|err| Error::invalid_boc(format!("can not read `master` from block: {}", err)))?
        .ok_or(Error::inappropriate_block(
            "not a masterchain block. Only key block contains blockchain configuration",
        ))?;

    let config = master.config().ok_or(Error::inappropriate_block(
        "not a key block. Only key block contains blockchain configuration",
    ))?;

    let cell = config
        .write_to_new_cell()
        .map_err(|err| Error::serialization_error(err, "config to cells"))?;

    let bytes = ton_types::serialize_toc(&cell.into())
        .map_err(|err| Error::serialization_error(err, "config cells to bytes"))?;

    Ok(ResultOfGetBlockchainConfig {
        config_boc: base64::encode(&bytes),
    })
}
