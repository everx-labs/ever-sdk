/*
 * Copyright 2018-2021 EverX Labs Ltd.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific EVERX DEV software governing permissions and
 * limitations under the License.
 *
 */

use super::stack::serialize_item;
use super::types::{ExecutionOptions, ResolvedExecutionOptions};
use crate::boc::internal::{
    deserialize_cell_from_boc, deserialize_object_from_boc, deserialize_object_from_cell,
    serialize_cell_to_boc, serialize_object_to_base64, serialize_object_to_boc,
    serialize_object_to_cell,
};
use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::processing::{parsing::decode_output, DecodedOutput};
use crate::tvm::{check_transaction::calc_transaction_fees, Error};
use crate::{abi::Abi, boc::BocCacheType};
use serde_json::Value;
use std::convert::TryFrom;
use std::sync::{atomic::AtomicU64, Arc};
use ever_block::{Account, CurrencyCollection, Message, MsgAddressInt, Serializable, Transaction};
use ever_executor::{
    ExecuteParams, ExecutorError, OrdinaryTransactionExecutor, TransactionExecutor,
};
use ever_sdk::TransactionFees;
use ever_block::{Cell, UInt256};
use ever_vm::stack::StackItem;

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

const UNLIMITED_BALANCE: u64 = u64::MAX;

impl AccountForExecutor {
    pub fn get_account(
        &self,
        context: &Arc<ClientContext>,
        address: MsgAddressInt,
    ) -> ClientResult<(Cell, Option<CurrencyCollection>)> {
        match self {
            AccountForExecutor::None => {
                let account = Account::default().serialize().unwrap();
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
                        deserialize_object_from_boc(context, &boc, "account")?.object;
                    let original_balance = account
                        .balance()
                        .ok_or_else(|| {
                            Error::invalid_account_boc(
                                "can not set unlimited balance for non existed account",
                            )
                        })?
                        .clone();
                    let mut balance = original_balance.clone();
                    balance.grams = UNLIMITED_BALANCE.into();
                    account.set_balance(balance);
                    let account = serialize_object_to_cell(&account, "account")?;
                    Ok((account, Some(original_balance)))
                } else {
                    let (_, account) = deserialize_cell_from_boc(context, &boc, "account")?;
                    Ok((account, None))
                }
            }
        }
    }

    pub fn restore_balance_if_needed(
        account: Cell,
        balance: Option<CurrencyCollection>,
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
    pub return_updated_account: Option<bool>,
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
    pub return_updated_account: Option<bool>,
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
    /// Attention! Only `account_state.storage.state.data` part of the BOC is updated.
    pub account: String,
}

fn parse_transaction(
    context: &Arc<ClientContext>,
    transaction: &Transaction,
) -> ClientResult<Value> {
    Ok(crate::boc::parse_transaction(
        context.clone(),
        crate::boc::ParamsOfParse {
            boc: serialize_object_to_base64(transaction, "transaction")?,
        },
    )?
    .parsed)
}

/// Emulates all the phases of contract execution locally
///
/// Performs all the phases of contract execution on Transaction Executor -
/// the same component that is used on Validator Nodes.
///
/// Can be used for contract debugging, to find out the reason why a message was not delivered successfully.
/// Validators throw away the failed external inbound messages (if they failed before `ACCEPT`) in the real network.
/// This is why these messages are impossible to debug in the real network.
/// With the help of run_executor you can do that. In fact, `process_message` function
/// performs local check with `run_executor` if there was no transaction as a result of processing
/// and returns the error, if there is one.
///
/// Another use case to use `run_executor` is to estimate fees for message execution.
/// Set  `AccountForExecutor::Account.unlimited_balance`
/// to `true` so that emulation will not depend on the actual balance.
/// This may be needed to calculate deploy fees for an account that does not exist yet.
/// JSON with fees is in `fees` field of the result.
///
/// One more use case - you can produce the sequence of operations,
/// thus emulating the sequential contract calls locally.
/// And so on.
///
/// Transaction executor requires account BOC (bag of cells) as a parameter.
/// To get the account BOC - use `net.query` method to download it from GraphQL API
/// (field `boc` of `account`) or generate it with `abi.encode_account` method.
///
/// Also it requires message BOC. To get the message BOC - use `abi.encode_message` or `abi.encode_internal_message`.
///
/// If you need this emulation to be as precise as possible (for instance - emulate transaction
/// with particular lt in particular block or use particular blockchain config,
/// downloaded from a particular key block - then specify `execution_options` parameter.
///
/// If you need to see the aborted transaction as a result, not as an error, set `skip_transaction_check` to `true`.

#[api_function]
pub async fn run_executor(
    context: Arc<ClientContext>,
    params: ParamsOfRunExecutor,
) -> ClientResult<ResultOfRunExecutor> {
    run_executor_internal(context, params, true).await
}

pub async fn run_executor_internal(
    context: Arc<ClientContext>,
    params: ParamsOfRunExecutor,
    show_tips_on_error: bool,
) -> ClientResult<ResultOfRunExecutor> {
    let message =
        deserialize_object_from_boc::<Message>(&context, &params.message, "message")?.object;
    let msg_address = message
        .dst_ref()
        .ok_or_else(|| Error::invalid_message_type())?
        .clone();
    let (account, _) = params.account.get_account(&context, msg_address.clone())?;
    let options =
        ResolvedExecutionOptions::from_options(&context, params.execution_options).await?;

    let account_copy = account.clone();
    let contract_info = move || async move {
        let account = deserialize_object_from_cell::<Account>(account_copy.clone(), "account")?;
        if let (Some(addr), Some(balance)) = (account.get_addr(), account.balance()) {
            Ok((addr.clone(), balance.grams.as_u128() as u64))
        } else {
            Ok((msg_address.clone(), 0))
        }
    };

    let (transaction, modified_account) = call_executor(
        account.clone(),
        message,
        options,
        contract_info.clone(),
        show_tips_on_error,
    )
    .await?;

    let sdk_transaction = ever_sdk::Transaction::try_from(&transaction)
        .map_err(|err| Error::can_not_read_transaction(err))?;

    let fees = calc_transaction_fees(
        &sdk_transaction,
        false,
        params.skip_transaction_check.unwrap_or_default(),
        contract_info,
        show_tips_on_error,
    )
    .await?;

    let mut out_messages = vec![];
    for i in 0..transaction.msg_count() {
        let message = transaction
            .get_out_msg(i)
            .map_err(|err| Error::can_not_read_transaction(err))?
            .ok_or_else(|| Error::can_not_read_transaction("message missing"))?;
        out_messages.push(serialize_object_to_base64(&message, "message")?);
    }

    // TODO decode Message object without converting to string
    let decoded = if let Some(abi) = params.abi.as_ref() {
        Some(decode_output(&context, abi, out_messages.clone())?)
    } else {
        None
    };

    let account = if params.return_updated_account.unwrap_or_default() {
        serialize_cell_to_boc(&context, modified_account, "account", params.boc_cache)?
    } else {
        String::new()
    };

    Ok(ResultOfRunExecutor {
        out_messages,
        transaction: parse_transaction(&context, &transaction)?,
        account,
        decoded,
        fees,
    })
}

/// Executes get-methods of ABI-compatible contracts
///
/// Performs only a part of compute phase of transaction execution
/// that is used to run get-methods of ABI-compatible contracts.
///
/// If you try to run get-methods with `run_executor` you will get an error, because it checks ACCEPT and exits
/// if there is none, which is actually true for get-methods.
///
///  To get the account BOC (bag of cells) - use `net.query` method to download it from GraphQL API
/// (field `boc` of `account`) or generate it with `abi.encode_account method`.
/// To get the message BOC - use `abi.encode_message` or prepare it any other way, for instance, with FIFT script.
///
/// Attention! Updated account state is produces as well, but only
/// `account_state.storage.state.data`  part of the BOC is updated.
#[api_function]
pub async fn run_tvm(
    context: Arc<ClientContext>,
    params: ParamsOfRunTvm,
) -> ClientResult<ResultOfRunTvm> {
    let mut account = deserialize_object_from_boc::<Account>(&context, &params.account, "account")?;
    let message =
        deserialize_object_from_boc::<Message>(&context, &params.message, "message")?.object;
    let options =
        ResolvedExecutionOptions::from_options(&context, params.execution_options).await?;
    if account.object.is_none() {
        return Err(Error::invalid_account_boc("Account is None"));
    }

    let messages = super::call_tvm::call_tvm_msg(&mut account.object, options, &message)?;

    let mut out_messages = vec![];
    for message in messages {
        out_messages.push(serialize_object_to_boc(
            &context,
            &message,
            "message",
            params.boc_cache.clone(),
        )?);
    }

    // TODO decode Message object without converting to string
    let decoded = if let Some(abi) = params.abi.as_ref() {
        Some(decode_output(&context, abi, out_messages.clone())?)
    } else {
        None
    };

    let account = if params.return_updated_account.unwrap_or_default() {
        serialize_object_to_boc(&context, &account.object, "account", params.boc_cache)?
    } else {
        String::new()
    };

    Ok(ResultOfRunTvm {
        out_messages,
        account,
        decoded,
    })
}

/// Executes get-methods of ABI-compatible contracts
///
/// Performs only a part of compute phase of transaction execution
/// that is used to run get-methods of ABI-compatible contracts.
///
/// If you try to run get-methods with `run_executor` you will get an error, because it checks ACCEPT and exits
/// if there is none, which is actually true for get-methods.
///
///  To get the account BOC (bag of cells) - use `net.query` method to download it from GraphQL API
/// (field `boc` of `account`) or generate it with `abi.encode_account method`.
/// To get the message BOC - use `abi.encode_message` or prepare it any other way, for instance, with FIFT script.
///
/// Attention! Updated account state is produces as well, but only
/// `account_state.storage.state.data`  part of the BOC is updated.
#[api_function]
pub async fn run_solidity_getter(
    context: Arc<ClientContext>,
    params: ParamsOfRunTvm,
    stack_items: Vec<StackItem>
) -> ClientResult<Vec<StackItem>> {
    let mut account = deserialize_object_from_boc::<Account>(&context, &params.account, "account")?;
    let options =
        ResolvedExecutionOptions::from_options(&context, params.execution_options).await?;
    if account.object.is_none() {
        return Err(Error::invalid_account_boc("Account is None"));
    }

    Ok(super::call_tvm::call_tvm_msg_getter(&mut account.object, options, stack_items)?)
}

async fn call_executor<F>(
    mut account_root: Cell,
    msg: Message,
    options: ResolvedExecutionOptions,
    contract_info: impl FnOnce() -> F,
    show_tips_on_error: bool,
) -> ClientResult<(Transaction, Cell)>
where
    F: futures::Future<Output = ClientResult<(MsgAddressInt, u64)>>,
{
    let executor = OrdinaryTransactionExecutor::new(
        Arc::try_unwrap(options.blockchain_config).unwrap_or_else(|arc| arc.as_ref().clone()),
    );
    let params = ExecuteParams {
        block_unixtime: options.block_time,
        block_lt: options.block_lt,
        last_tr_lt: Arc::new(AtomicU64::new(options.transaction_lt)),
        behavior_modifiers: Some(options.behavior_modifiers),
        seed_block: UInt256::rand(),
        signature_id: options.signature_id,
        ..ExecuteParams::default()
    };
    let msg = ever_block::CommonMessage::Std(msg);
    let transaction =
        match executor.execute_with_libs_and_params(Some(&msg), &mut account_root, params) {
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
                            Error::tvm_execution_failed(
                                err_message,
                                *code,
                                exit_arg,
                                &address,
                                None,
                                show_tips_on_error,
                            )
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

    Ok((transaction, account_root))
}
