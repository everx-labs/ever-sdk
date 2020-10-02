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

use crate::error::{ApiError, ApiResult};
use crate::client::ClientContext;
use crate::dispatch::DispatchTable;
use crate::boc::{deserialize_cell_from_base64, deserialize_object_from_base64};
use ton_executor::BlockchainConfig;
use std::convert::{TryFrom, TryInto};

use errors::Error;

mod check_transaction;
mod errors;
mod executor;
mod get;
mod tvm;

#[derive(Clone, Default)]
pub(crate) struct ExecutionOptionsInternal {
    pub blockchain_config: Option<BlockchainConfig>,
    pub block_unixtime: Option<u32>,
    pub block_lt: Option<u64>,
    pub transaction_lt: Option<u64>,
}

pub(crate) fn blockchain_config_from_base64(b64: &str) -> ApiResult<BlockchainConfig> {
    let config_params = deserialize_object_from_base64(b64, "blockchain config")?;
    BlockchainConfig::with_config(config_params.object)
        .map_err(|err| Error::can_not_read_blockchain_config(err))
}

impl TryFrom<ExecutionOptions> for ExecutionOptionsInternal {
    type Error = ApiError;
    fn try_from(options: ExecutionOptions) -> ApiResult<Self> {
        Ok(ExecutionOptionsInternal {
            block_lt: options.block_lt,
            block_unixtime: options.block_unixtime,
            transaction_lt: options.transaction_lt,
            blockchain_config: options.blockchain_config
                .map(|string| blockchain_config_from_base64(&string))
                .transpose()?,
        })
    }
}

#[derive(Serialize, Deserialize, TypeInfo, Clone, Default)]
pub struct ExecutionOptions {
    /// boc with config
    pub blockchain_config: Option<String>,
    /// time that is used as transaction time
    pub block_unixtime: Option<u32>,
    /// block logical time
    pub block_lt: Option<u64>,
    /// transaction logical time
    pub transaction_lt: Option<u64>,
}

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
pub enum ExecutionMode {
    /// Executes all phases and performs all checks
    Full,
    /// Executes contract only on TVM (part of compute phase)
    TvmOnly,
}

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
pub struct ParamsOfExecute {
    /// Account BOC. Must be encoded as base64
    pub account: String,
    /// Input message BOC for the contract. Must be encoded as base64
    pub in_message: String,
    /// Execution mode
    pub mode: ExecutionMode,
    /// Execution options
    pub execution_options: Option<ExecutionOptions>,
}

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
pub struct ResultOfExecute {
    /// JSON with parsed updated account state. Attention! When used in `TvmOnly` mode only data in account state is updated
    pub account: serde_json::Value,
    /// Array of parsed output messages
    pub out_messages: Vec<serde_json::Value>,
    /// JSON with parsed transaction, returned only in case of `Full` mode execution
    pub transaction: Option<serde_json::Value>,
}

pub async fn execute(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfExecute,
) -> ApiResult<ResultOfExecute> {
    let (_, account_cell) = deserialize_cell_from_base64(&params.account, "account")?;
    let message = deserialize_object_from_base64::<ton_block::Message>(
        &params.in_message, "message"
    )?;

    let options: Option<ExecutionOptionsInternal> = params.execution_options
        .map(|options| options.try_into())
        .transpose()?;

    // match params.mode {
    //     ExecutionMode::Full => executor::execute(),
    //     ExecutionMode::TvmOnly => tvm::execute(),
    // }

    Ok(ResultOfExecute {
        account: serde_json::json!({}),
        out_messages: vec![],
        transaction: None,
    })
}

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.spawn("tvm.execute", execute);
}
