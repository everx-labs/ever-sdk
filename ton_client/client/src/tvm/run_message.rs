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

use crate::{
    abi::Abi,
    boc::internal::{
        deserialize_cell_from_base64, deserialize_object_from_base64, deserialize_object_from_cell,
        serialize_cell_to_base64, serialize_object_to_base64,
    },
    client::ClientContext,
    error::ClientResult,
    processing::{parsing::decode_output, DecodedOutput},
    tvm::{check_transaction::check_transaction, Error},
};
use super::types::{ExecutionOptions, ResolvedExecutionOptions};
use num_traits::ToPrimitive;
use serde_json::Value;
use std::sync::{Arc, atomic::AtomicU64};
use ton_block::{Message, Serializable};
use ton_executor::{ExecutorError, OrdinaryTransactionExecutor, TransactionExecutor};
use ton_sdk::TransactionFees;
use ton_types::Cell;

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfRunExecutor {
    /// Input message BOC. Must be encoded as base64.
    pub message: String,
    /// Account BOC. Must be encoded as base64.
    pub account: Option<String>,
    /// Execution options.
    pub execution_options: Option<ExecutionOptions>,
    /// Contract ABI for dedcoding output messages
    pub abi: Option<Abi>,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfRunTvm {
    /// Input message BOC. Must be encoded as base64.
    pub message: String,
    /// Account BOC. Must be encoded as base64.
    pub account: String,
    /// Execution options.
    pub execution_options: Option<ExecutionOptions>,
    /// Contract ABI for dedcoding output messages
    pub abi: Option<Abi>,
}

#[derive(Serialize, Deserialize, ApiType, Debug, PartialEq, Clone)]
pub struct ResultOfRunExecutor {
    /// Parsed transaction.
    ///
    /// In addition to the regular transaction fields there is a
    /// `boc` field encoded with `base64` which contains source
    /// transaction BOC.
    pub transaction: Value,

    /// List of output messages' BOCs. Encoded as `base64`
    pub out_messages: Vec<String>,

    /// Optional decoded message bodies according to the optional
    /// `abi` parameter.
    pub decoded: Option<DecodedOutput>,

    /// Updated account state BOC. Encoded as `base64`
    pub account: String,

    /// Transaction fees
    pub fees: TransactionFees,
}

#[derive(Serialize, Deserialize, ApiType, Debug, PartialEq, Clone)]
pub struct ResultOfRunTvm {
    /// List of output messages' BOCs. Encoded as `base64`
    pub out_messages: Vec<String>,

    /// Optional decoded message bodies according to the optional
    /// `abi` parameter.
    pub decoded: Option<DecodedOutput>,

    /// Updated account state BOC. Encoded as `base64`.
    /// Attention! Only data in account state is updated.
    pub account: String,
}

fn parse_transaction(
    context: &Arc<ClientContext>,
    transaction: &ton_block::Transaction,
) -> ClientResult<Value> {
    Ok(crate::boc::parse_transaction(
        context.clone(),
        crate::boc::ParamsOfParse {
            boc: serialize_object_to_base64(transaction, "transaction")?,
        },
    )?
    .parsed)
}

#[api_function]
pub async fn run_executor(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfRunExecutor,
) -> ClientResult<ResultOfRunExecutor> {
    let account = params
        .account
        .map(|string| deserialize_cell_from_base64(&string, "account"))
        .transpose()?
        .map(|tuple| tuple.1);
    let message = deserialize_object_from_base64::<Message>(&params.message, "message")?.object;
    let options = ResolvedExecutionOptions::from_options(&context, params.execution_options)?;

    // if no account provided use AccountNone
    let account = match account {
        Some(account) => account,
        None => ton_block::Account::AccountNone
            .write_to_new_cell()
            .unwrap()
            .into(),
    };

    let account_copy = account.clone();
    let msg_address = message.dst().ok_or(Error::invalid_message_type())?;
    let contract_info = move || async move {
        let account: ton_block::Account =
            deserialize_object_from_cell(account_copy.clone(), "account")?;
        match account.stuff() {
            Some(stuff) => {
                let balance = stuff
                    .storage
                    .balance
                    .grams
                    .value()
                    .to_u64()
                    .unwrap_or_default();
                Ok((stuff.addr.clone(), balance))
            }
            None => Ok((msg_address.clone(), 0)),
        }
    };

    let (transaction, account) = call_executor(account, message, options, contract_info.clone()).await?;

    let fees = check_transaction(&transaction, false, contract_info).await?;

    let mut out_messages = vec![];
    for i in 0..transaction.outmsg_cnt {
        let message = transaction
            .get_out_msg(i)
            .map_err(|err| Error::can_not_read_transaction(err))?
            .ok_or(Error::can_not_read_transaction("message missing"))?;
        out_messages.push(serialize_object_to_base64(&message, "message")?);
    }

    // TODO decode Message object without converting to string
    let decoded = if let Some(abi) = params.abi.as_ref() {
        Some(decode_output(&context, abi, out_messages.clone())?)
    } else {
        None
    };

    Ok(ResultOfRunExecutor {
        out_messages,
        transaction: parse_transaction(&context, &transaction)?,
        account: serialize_cell_to_base64(&account, "account")?,
        decoded,
        fees,
    })
}

#[api_function]
pub async fn run_tvm(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfRunTvm,
) -> ClientResult<ResultOfRunTvm> {
    let account = deserialize_object_from_base64(&params.account, "account")?.object;
    let message = deserialize_object_from_base64::<Message>(&params.message, "message")?.object;
    let options = ResolvedExecutionOptions::from_options(&context, params.execution_options)?;

    let stuff = match account {
        ton_block::Account::AccountNone => Err(Error::invalid_account_boc("Acount is None")),
        ton_block::Account::Account(stuff) => Ok(stuff),
    }?;

    let (messages, stuff) = super::call_tvm::call_tvm_msg(stuff, options, &message)?;

    let mut out_messages = vec![];
    for message in messages {
        out_messages.push(serialize_object_to_base64(&message, "message")?);
    }

    // TODO decode Message object without converting to string
    let decoded = if let Some(abi) = params.abi.as_ref() {
        Some(decode_output(&context, abi, out_messages.clone())?)
    } else {
        None
    };

    Ok(ResultOfRunTvm {
        out_messages,
        account: serialize_object_to_base64(&ton_block::Account::Account(stuff), "account")?,
        decoded,
    })
}

async fn call_executor<F>(
    mut account: Cell,
    msg: ton_block::Message,
    options: ResolvedExecutionOptions,
    contract_info: impl FnOnce() -> F,
) -> ClientResult<(ton_block::Transaction, Cell)> 
where 
    F: futures::Future<Output=ClientResult<(ton_block::MsgAddressInt, u64)>>
{
    let executor = OrdinaryTransactionExecutor::new(options.blockchain_config);
    let result = executor
        .execute(
            Some(&msg),
            &mut account,
            options.block_time,
            options.block_lt,
            Arc::new(AtomicU64::new(options.transaction_lt)),
            false,
        );

    let transaction = match result {
        Ok(transaction) => transaction,
        Err(err) => {
            let err_message = err.to_string();
            let err = match contract_info().await {
                Ok((address, balance)) => match &err.downcast_ref::<ExecutorError>() {
                    Some(ExecutorError::NoAcceptError(code)) => {
                        Error::tvm_execution_failed(err_message, *code, None, &address)
                    }
                    Some(ExecutorError::NoFundsToImportMsg) => {
                        Error::low_balance(&address, balance)
                    }
                    _ => Error::unknown_execution_error(err),
                },
                Err(err) => err,
            };
            return Err(err);
        }
    };

    Ok((transaction, account))
}
