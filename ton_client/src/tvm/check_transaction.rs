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

use super::errors::Error;
use crate::error::ClientResult;
use ever_block::AccStatusChange;
use ton_sdk::Transaction;

pub(crate) async fn calc_transaction_fees<F>(
    transaction: &Transaction,
    real_tr: bool,
    skip_check: bool,
    contract_info: impl FnOnce() -> F,
    show_tips_on_error: bool,
) -> ClientResult<ton_sdk::TransactionFees>
where
    F: futures::Future<Output = ClientResult<(ever_block::MsgAddressInt, u64)>>,
{
    if !transaction.is_aborted() || skip_check {
        return Ok(transaction.calc_fees());
    }

    let mut error = match extract_error(&transaction, contract_info, show_tips_on_error).await {
        Err(err) => err,
        Ok(_) => Error::transaction_aborted(),
    };

    if real_tr {
        error.data["transaction_id"] = transaction.id().to_string().into()
    }

    Err(error)
}

pub(crate) async fn extract_error<F>(
    transaction: &Transaction,
    contract_info: impl FnOnce() -> F,
    show_tips: bool,
) -> ClientResult<()>
where
    F: futures::Future<Output = ClientResult<(ever_block::MsgAddressInt, u64)>>,
{
    if let Some(storage) = &transaction.storage {
        if storage.status_change != AccStatusChange::Unchanged {
            let (address, balance) = contract_info().await?;
            return Err(Error::storage_phase_failed(
                &storage.status_change,
                &address,
                balance,
            ));
        }
    }

    if let Some(reason) = &transaction.compute.skipped_reason {
        let (address, balance) = contract_info().await?;
        return Err(Error::tvm_execution_skipped(&reason, &address, balance));
    }

    if transaction.compute.success.is_none() || !transaction.compute.success.unwrap() {
        let (address, _) = contract_info().await?;
        return Err(Error::tvm_execution_failed(
            "Compute phase isn't succeeded",
            transaction.compute.exit_code.unwrap_or(-1),
            transaction.compute.exit_arg.map(i32::into),
            &address,
            Some(transaction.compute.gas_used),
            show_tips,
        ));
    }

    if let Some(action) = &transaction.action {
        if !action.success {
            let (address, balance) = contract_info().await?;
            return Err(Error::action_phase_failed(
                action.result_code,
                action.valid,
                action.no_funds,
                &address,
                balance,
            ));
        }
    }

    Ok(())
}
