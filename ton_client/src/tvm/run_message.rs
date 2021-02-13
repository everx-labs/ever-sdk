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

use super::stack::serialize_item;
use super::types::{ExecutionOptions, ResolvedExecutionOptions};
use crate::{abi::Abi, boc::BocCacheType};
use crate::boc::internal::{
    deserialize_cell_from_boc, deserialize_object_from_boc, deserialize_object_from_cell,
    serialize_cell_to_boc, serialize_object_to_base64, serialize_object_to_boc,
    serialize_object_to_cell
};
use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::processing::{parsing::decode_output, DecodedOutput};
use crate::tvm::{check_transaction::calc_transaction_fees, Error};
use num_traits::ToPrimitive;
use serde_json::Value;
use std::sync::{atomic::AtomicU64, Arc};
use ton_block::{Account, Message, Serializable};
use ton_executor::{ExecutorError, OrdinaryTransactionExecutor, TransactionExecutor};
use ton_sdk::TransactionFees;
use ton_types::Cell;

#[derive(Serialize, Deserialize, ApiType, Debug, Clone)]
#[serde(tag = "type")]
pub enum AccountForExecutor {
    /// Non-existing account to run a creation internal message.
    /// Should be used with `skip_transaction_check = true` if the message has no deploy data
    /// since transactions on the uninitialized account are always aborted
    None,
    /// Emulate uninitialized account to run deploy message
    Uninit,
    /// Account state to run message
    Account {
        /// Account BOC. Encoded as base64.
        boc: String,
        /// Flag for running account with the unlimited balance. Can be used to calculate
        /// transaction fees without balance check
        unlimited_balance: Option<bool>,
    },
}

impl Default for AccountForExecutor {
    fn default() -> Self {
        AccountForExecutor::None
    }
}

const UNLIMITED_BALANCE: u64 = std::u64::MAX;

impl AccountForExecutor {
    pub async fn get_account(
        &self,
        context: &Arc<ClientContext>,
        address: ton_block::MsgAddressInt,
    ) -> ClientResult<(Cell, Option<ton_block::CurrencyCollection>)> {
        match self {
            AccountForExecutor::None => {
                let account = Account::AccountNone.write_to_new_cell().unwrap().into();
                Ok((account, None))
            }
            AccountForExecutor::Uninit => {
                let last_paid = (context.env.now_ms() / 1000) as u32;
                let account = Account::uninit(address, 0, last_paid, UNLIMITED_BALANCE.into());
                let account = serialize_object_to_cell(&account, "account")?;
                Ok((account, None))
            }
            AccountForExecutor::Account {
                boc,
                unlimited_balance,
            } => {
                if unlimited_balance.unwrap_or_default() {
                    let mut account: Account =
                        deserialize_object_from_boc(context, &boc, "account").await?.object;
                    let original_balance = account
                        .balance()
                        .ok_or_else(|| Error::invalid_account_boc(
                            "can not set unlimited balance for non existed account",
                        ))?
                        .clone();
                    let mut balance = original_balance.clone();
                    balance.grams = UNLIMITED_BALANCE.into();
                    account.set_balance(balance);
                    let account = serialize_object_to_cell(&account, "account")?;
                    Ok((account, Some(original_balance)))
                } else {
                    let (_, account) = deserialize_cell_from_boc(context, &boc, "account").await?;
                    Ok((account, None))
                }
            }
        }
    }

    pub fn restore_balance_if_needed(
        account: Cell,
        balance: Option<ton_block::CurrencyCollection>,
    ) -> ClientResult<Cell> {
        if let Some(balance) = balance {
            let mut account: Account = deserialize_object_from_cell(account, "account")?;
            account.set_balance(balance);
            serialize_object_to_cell(&account, "account")
        } else {
            Ok(account)
        }
    }
}

#[derive(Serialize, Deserialize, ApiType, Clone, Default)]
pub struct ParamsOfRunExecutor {
    /// Input message BOC. Must be encoded as base64.
    pub message: String,
    /// Account to run on executor
    pub account: AccountForExecutor,
    /// Execution options.
    pub execution_options: Option<ExecutionOptions>,
    /// Contract ABI for decoding output messages
    pub abi: Option<Abi>,
    /// Skip transaction check flag
    pub skip_transaction_check: Option<bool>,
    /// Cache type to put the result. The BOC itself returned if no cache type provided
    pub boc_cache: Option<BocCacheType>,
    /// Return updated account flag. Empty string is returned if the flag is `false`
    pub return_updated_account: Option<bool>
}

#[derive(Serialize, Deserialize, ApiType, Clone, Default)]
pub struct ParamsOfRunTvm {
    /// Input message BOC. Must be encoded as base64.
    pub message: String,
    /// Account BOC. Must be encoded as base64.
    pub account: String,
    /// Execution options.
    pub execution_options: Option<ExecutionOptions>,
    /// Contract ABI for decoding output messages
    pub abi: Option<Abi>,
    /// Cache type to put the result. The BOC itself returned if no cache type provided
    pub boc_cache: Option<BocCacheType>,
    /// Return updated account flag. Empty string is returned if the flag is `false`
    pub return_updated_account: Option<bool>
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug, PartialEq, Clone)]
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

#[derive(Serialize, Deserialize, ApiType, Default, Debug, PartialEq, Clone)]
pub struct ResultOfRunTvm {
    /// List of output messages' BOCs. Encoded as `base64`
    pub out_messages: Vec<String>,

    /// Optional decoded message bodies according to the optional
    /// `abi` parameter.
    pub decoded: Option<DecodedOutput>,

    /// Updated account state BOC. Encoded as `base64`.
    /// Attention! Only `account_state.storage.state.data` part of the boc is updated. 
    pub account: String,
}

async fn parse_transaction(
    context: &Arc<ClientContext>,
    transaction: &ton_block::Transaction,
) -> ClientResult<Value> {
    Ok(crate::boc::parse_transaction(
        context.clone(),
        crate::boc::ParamsOfParse {
            boc: serialize_object_to_base64(transaction, "transaction")?,
        },
    )
    .await?
    .parsed)
}

/// Emulates all the phases of contract execution locally
/// 
/// Performs all the phases of contract execution on Transaction Executor - 
/// the same component that is used on Validator Nodes. 
/// 
/// Can be used for contract debug, to find out the reason of message unsuccessful 
/// delivery - as Validators just throw away failed transactions, here you can catch it. 
/// 
/// Another use case is to estimate fees for message execution. Set  `AccountForExecutor::Account.unlimited_balance`
/// to `true` so that emulation will not depend on the actual balance.
/// 
/// One more use case - you can produce the sequence of operations,
/// thus emulating the multiple contract calls locally. 
/// And so on. 
/// 
/// To get the account boc (bag of cells) - use `net.query` method to download it from graphql api
/// (field `boc` of `account`) or generate it with `abi.encode_account method`. 
/// To get the message boc - use `abi.encode_message` or prepare it any other way, for instance, with Fift script.
/// 
/// If you need this emulation to be as precise as possible then specify `ParamsOfRunExecutor` parameter.
/// If you need to see the aborted transaction as a result, not as an error, set `skip_transaction_check` to `true`.

#[api_function]
pub async fn run_executor(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfRunExecutor,
) -> ClientResult<ResultOfRunExecutor> {
    let message = deserialize_object_from_boc::<Message>(&context, &params.message, "message").await?.object;
    let msg_address = message.dst().ok_or_else(|| Error::invalid_message_type())?;
    let (account, _) = params.account.get_account(&context, msg_address.clone()).await?;
    let options = ResolvedExecutionOptions::from_options(&context, params.execution_options).await?;

    let account_copy = account.clone();
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

    let (transaction, modified_account) =
        call_executor(account.clone(), message, options, contract_info.clone()).await?;

    let fees = calc_transaction_fees(
        &transaction,
        false,
        params.skip_transaction_check.unwrap_or_default(),
        contract_info,
    )
    .await?;

    let mut out_messages = vec![];
    for i in 0..transaction.outmsg_cnt {
        let message = transaction
            .get_out_msg(i)
            .map_err(|err| Error::can_not_read_transaction(err))?
            .ok_or_else(|| Error::can_not_read_transaction("message missing"))?;
        out_messages.push(serialize_object_to_base64(&message, "message")?);
    }

    // TODO decode Message object without converting to string
    let decoded = if let Some(abi) = params.abi.as_ref() {
        Some(decode_output(&context, abi, out_messages.clone()).await?)
    } else {
        None
    };

    let account = if params.return_updated_account.unwrap_or_default() {
        serialize_cell_to_boc(&context, modified_account, "account", params.boc_cache).await?
    } else {
        String::new()
    };

    Ok(ResultOfRunExecutor {
        out_messages,
        transaction: parse_transaction(&context, &transaction).await?,
        account,
        decoded,
        fees,
    })
}


/// Executes get methods of ABI-compatible contracts
/// 
/// Performs only a part of compute phase of transaction execution
/// that is used to run get-methods of ABI-compatible contracts.
///  
/// If you try to run get methods with `run_executor` you will get an error, because it checks ACCEPT and exits
/// if there is none, which is actually true for get methods. 
///
///  To get the account boc (bag of cells) - use `net.query` method to download it from graphql api
/// (field `boc` of `account`) or generate it with `abi.encode_account method`. 
/// To get the message boc - use `abi.encode_message` or prepare it any other way, for instance, with Fift script.
/// 
/// Attention! Updated account state is produces as well, but only 
/// `account_state.storage.state.data`  part of the boc is updated. 
#[api_function]
pub async fn run_tvm(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfRunTvm,
) -> ClientResult<ResultOfRunTvm> {
    let account = deserialize_object_from_boc(&context, &params.account, "account").await?;
    let message = deserialize_object_from_boc::<Message>(&context, &params.message, "message").await?.object;
    let options = ResolvedExecutionOptions::from_options(&context, params.execution_options).await?;
    let stuff = match account.object {
        ton_block::Account::AccountNone => Err(Error::invalid_account_boc("Acount is None")),
        ton_block::Account::Account(stuff) => Ok(stuff),
    }?;

    let (messages, stuff) = super::call_tvm::call_tvm_msg(stuff, options, &message)?;

    let mut out_messages = vec![];
    for message in messages {
        out_messages.push(serialize_object_to_boc(&context, &message, "message", params.boc_cache.clone()).await?);
    }

    // TODO decode Message object without converting to string
    let decoded = if let Some(abi) = params.abi.as_ref() {
        Some(decode_output(&context, abi, out_messages.clone()).await?)
    } else {
        None
    };

    let account = if params.return_updated_account.unwrap_or_default() {
        serialize_object_to_boc(&context, &ton_block::Account::Account(stuff), "account", params.boc_cache).await?
    } else {
        String::new()
    };

    Ok(ResultOfRunTvm {
        out_messages,
        account,
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
    F: futures::Future<Output = ClientResult<(ton_block::MsgAddressInt, u64)>>,
{
    let executor = OrdinaryTransactionExecutor::new(options.blockchain_config);
    let result = executor.execute(
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
                    Some(ExecutorError::NoAcceptError(code, exit_arg)) => {
                        let exit_arg = exit_arg
                            .as_ref()
                            .map(|item| serialize_item(item))
                            .transpose()?;
                        Error::tvm_execution_failed(err_message, *code, exit_arg, &address)
                    }
                    Some(ExecutorError::NoFundsToImportMsg) => {
                        Error::low_balance(&address, balance)
                    }
                    Some(ExecutorError::ExtMsgComputeSkipped(reason)) => {
                        Error::tvm_execution_skipped(reason, &address, balance)
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
