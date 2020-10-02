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

use super::ExecutionOptionsInternal;
use super::errors::Error;
use super::check_transaction::check_transaction_status;
use crate::error::ApiResult;
use crate::client::ClientContext;
use ton_executor::{ExecutorError, BlockchainConfig, OrdinaryTransactionExecutor, TransactionExecutor};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

pub(crate) fn execute(
    context: &Arc<ClientContext>,
    account: ton_types::Cell,
    msg: ton_block::Message,
    options: Option<ExecutionOptionsInternal>,
) -> ApiResult<(ton_block::Transaction, ton_types::Cell)> {
    let options = options.unwrap_or_default();

    let account_copy = account.clone();
    let contract_info = move || {
        let account = ton_sdk::Contract::from_cells(account_copy.clone().into())
            .map_err(|err| crate::boc::Error::invalid_boc(format!("Can not read account data: {}", err)))?;
        
        Ok((account.address(), account.balance))
    };

    let (transaction, account) = call_executor(
        account,
        msg,
        options.blockchain_config.unwrap_or_default(),
        options.block_unixtime.unwrap_or_else(|| (context.env.now_ms() / 1000) as u32),
        options.block_lt,
        options.transaction_lt,
        &contract_info
    )?;

    check_transaction_status(&transaction, false, &contract_info)?;

    Ok((transaction, account))
}


pub(crate) fn call_executor(
    mut account: ton_types::Cell,
    msg: ton_block::Message,
    config: BlockchainConfig,
    timestamp: u32,
    block_lt: Option<u64>,
    transaction_lt: Option<u64>,
    contract_info: impl FnOnce() -> ApiResult<(ton_block::MsgAddressInt, u64)>,
) -> ApiResult<(ton_block::Transaction, ton_types::Cell)> {
    let block_lt = block_lt.unwrap_or(transaction_lt.unwrap_or(1_000_001) - 1);
    let lt = Arc::new(AtomicU64::new(transaction_lt.unwrap_or(block_lt + 1)));
    let executor = OrdinaryTransactionExecutor::new(config);
    let transaction = executor.execute(
        Some(&msg),
        &mut account,
        timestamp,
        block_lt,
        lt.clone(),
        false)
        .map_err(|err| {
            match contract_info() {
                Ok((address, balance)) => {
                    match err.downcast_ref::<ExecutorError>() {
                        Some(ExecutorError::NoAcceptError(code)) => Error::tvm_execution_failed(*code, &address),
                        Some(ExecutorError::NoFundsToImportMsg) => Error::low_balance(&address, balance),
                        _ => Error::unknown_execution_error(err)
                    }
                },
                Err(err) => err
            }
        })?;
    Ok((transaction, account))
}
