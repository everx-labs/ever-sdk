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

use super::check_transaction::check_transaction_status;
use super::errors::Error;
use crate::boc::internal::deserialize_object_from_base64;
use crate::client::ClientContext;
use crate::error::ApiResult;
use crate::tvm::execute_message::{ExecutionOptions, ExecutionOptionsInternal};
use crate::tvm::execute_message_full::execute_message_full;
use failure::_core::convert::TryFrom;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use ton_executor::{
    BlockchainConfig, ExecutorError, OrdinaryTransactionExecutor, TransactionExecutor,
};
use ton_types::SliceData;

#[derive(Clone, Default)]
pub(crate) struct ExecutionOptionsInternal {
    pub blockchain_config: Option<BlockchainConfig>,
    pub block_time: Option<u32>,
    pub block_lt: Option<u64>,
    pub transaction_lt: Option<u64>,
}

pub(crate) fn blockchain_config_from_base64(b64: &str) -> ApiResult<BlockchainConfig> {
    let config_params = deserialize_object_from_base64(b64, "blockchain config")?;
    BlockchainConfig::with_config(config_params.object)
        .map_err(|err| Error::can_not_read_blockchain_config(err))
}

pub(crate) fn execute_message_tvm_only(
    context: &Arc<ClientContext>,
    account: ton_types::Cell,
    message: ton_block::Message,
    options: Option<ExecutionOptionsInternal>,
) -> ApiResult<(ton_block::Transaction, ton_types::Cell)> {
    let contract = ton_sdk::Contract::from_cells(SliceData::from(account))
        .map_err(|err| Error::invalid_account_boc(err))?;
    let messages = contract
        .local_call_tvm(message)
        .map_err(|err| Error::execution_failed(err))?;
}
