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

use crate::boc::internal::{
    deserialize_cell_from_base64, deserialize_object_from_base64, deserialize_object_from_cell,
    serialize_cell_to_base64, serialize_object_to_base64,
};
use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::tvm::Error;
use ton_executor::{
    BlockchainConfig, ExecutorError, OrdinaryTransactionExecutor, TransactionExecutor,
};

use crate::abi::Abi;
use crate::processing::DecodedOutput;
use serde_json::Value;

#[derive(Serialize, Deserialize, ApiType, Clone, Default)]
pub struct ExecutionOptions {
    /// boc with config
    pub blockchain_config: Option<String>,
    /// time that is used as transaction time
    pub block_time: Option<u32>,
    /// block logical time
    pub block_lt: Option<u64>,
    /// transaction logical time
    pub transaction_lt: Option<u64>,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub enum ExecutionMode {
    /// Executes all phases and performs all checks
    Full,
    /// Executes contract only on TVM (part of compute phase)
    TvmOnly,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfExecuteMessage {
    /// Input message.
    pub message: MessageSource,
    /// Account BOC. Must be encoded as base64.
    pub account: Option<String>,
    /// Execution mode.
    pub mode: ExecutionMode,
    /// Execution options.
    pub execution_options: Option<ExecutionOptions>,
}

#[derive(Serialize, Deserialize, ApiType, Debug, PartialEq, Clone)]
pub struct ResultOfExecuteMessage {
    /// Parsed transaction.
    ///
    /// In addition to the regular transaction fields there is a
    /// `boc` field encoded with `base64` which contains source
    /// transaction BOC.
    pub transaction: Option<Value>,

    /// List of parsed output messages.
    ///
    /// Similar to the `transaction` each message contains the `boc`
    /// field.
    pub out_messages: Vec<Value>,

    /// Optional decoded message bodies according to the optional
    /// `abi` parameter.
    pub decoded: Option<DecodedOutput>,

    /// JSON with parsed updated account state. Attention! When used in
    /// `TvmOnly` mode only data in account state is updated.
    pub account: Value,
}

use crate::abi::MessageSource;
use crate::boc::{parse_account, parse_message, parse_transaction, ParamsOfParse, ResultOfParse};
use crate::processing::parsing::decode_output;
use crate::tvm::check_transaction::check_transaction_status;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use ton_block;
use ton_block::{Deserializable, Message, Serializable, Transaction};
use ton_vm::stack::{integer::IntegerData, savelist::SaveList, Stack, StackItem};
use ton_types::{Cell, SliceData};
use ton_types::dictionary::HashmapType;
use ton_vm::executor::gas::gas_state::Gas;
use num_traits::ToPrimitive;

struct ExecutionOutput {
    transaction: Option<Transaction>,
    messages: Option<Vec<Message>>,
    account: Cell,
}

fn parse_object<S: Serializable>(
    context: &Arc<ClientContext>,
    object: &S,
    name: &str,
    parser: fn(Arc<ClientContext>, ParamsOfParse) -> ClientResult<ResultOfParse>,
) -> ClientResult<Value> {
    Ok(parser(
        context.clone(),
        ParamsOfParse {
            boc: serialize_object_to_base64(object, name)?,
        },
    )?
    .parsed)
}

impl ExecutionOutput {
    fn with_transaction(transaction: Transaction, account: Cell) -> Self {
        Self {
            transaction: Some(transaction),
            messages: None,
            account
        }
    }

    fn with_messages(messages: Vec<Message>, account: Cell) -> Self {
        Self {
            transaction: None,
            messages: Some(messages),
            account,
        }
    }

    fn convert_to_result(
        self,
        context: &Arc<ClientContext>,
        abi: Option<&Abi>,
    ) -> ClientResult<ResultOfExecuteMessage> {
        let out_messages = self.convert_out_messages(context)?;
        let decoded = if let Some(abi) = abi {
            Some(decode_output(context, abi, &out_messages)?)
        } else {
            None
        };
        Ok(ResultOfExecuteMessage {
            out_messages,
            transaction: self.convert_transaction(context)?,
            account: self.convert_account(context)?,
            decoded,
        })
    }

    fn convert_transaction(&self, context: &Arc<ClientContext>) -> ClientResult<Option<Value>> {
        self.transaction
            .as_ref()
            .map(|x| parse_object(context, x, "transaction", parse_transaction))
            .transpose()
    }

    fn convert_account(&self, context: &Arc<ClientContext>) -> ClientResult<Value> {
        parse_account(
            context.clone(),
            ParamsOfParse {
                boc: serialize_cell_to_base64(&self.account, "account")?,
            },
        )
        .map(|parsed| parsed.parsed)
    }

    fn convert_out_messages(&self, context: &Arc<ClientContext>) -> ClientResult<Vec<Value>> {
        let mut out_messages = Vec::new();
        if let Some(messages) = &self.messages {
            for message in messages {
                out_messages.push(parse_object(context, message, "message", parse_message)?);
            }
        } else if let Some(transaction) = &self.transaction {
            for i in 0..transaction.outmsg_cnt {
                let message = transaction
                    .get_out_msg(i)
                    .map_err(|err| Error::can_not_read_transaction(err))?
                    .ok_or(Error::can_not_read_transaction("message missing"))?;
                out_messages.push(parse_object(context, &message, "message", parse_message)?);
            }
        }
        Ok(out_messages)
    }
}

pub(crate) struct ResolvedExecutionOptions {
    pub blockchain_config: BlockchainConfig,
    pub block_time: u32,
    pub block_lt: u64,
    pub transaction_lt: u64,
}

impl ResolvedExecutionOptions {
    pub fn from_options(
        context: &Arc<ClientContext>,
        options: Option<ExecutionOptions>
    ) -> ClientResult<Self> {
        let options = options.unwrap_or_default();

        let config = if let Some(config) = options.blockchain_config {
            blockchain_config_from_base64(&config)?
        } else {
            Default::default()
        };

        let block_lt = options.block_lt.unwrap_or(options.transaction_lt.unwrap_or(1_000_001) - 1);
        let transaction_lt = options.transaction_lt.unwrap_or(block_lt + 1);
        let block_time = options
            .block_time
            .unwrap_or_else(|| (context.env.now_ms() / 1000) as u32);

        Ok(Self {
            block_lt,
            block_time,
            blockchain_config: config,
            transaction_lt,
        })
    }
}

pub(crate) fn blockchain_config_from_base64(b64: &str) -> ClientResult<BlockchainConfig> {
    let config_params = deserialize_object_from_base64(b64, "blockchain config")?;
    BlockchainConfig::with_config(config_params.object)
        .map_err(|err| Error::can_not_read_blockchain_config(err))
}

#[api_function]
pub async fn execute_message(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfExecuteMessage,
) -> ClientResult<ResultOfExecuteMessage> {
    let account = params.account
        .map(|string| deserialize_cell_from_base64(&string, "account"))
        .transpose()?
        .map(|tuple| tuple.1);
    let (message, abi) = params.message.encode(&context).await?;
    let message = deserialize_object_from_base64::<ton_block::Message>(&message, "message")?.object;
    let options = ResolvedExecutionOptions::from_options(&context, params.execution_options)?;
    let output = match params.mode {
        ExecutionMode::Full => {
            execute_message_full(&context, account, message, options)
        }
        ExecutionMode::TvmOnly => {
            let account = account
                .ok_or(Error::invalid_account_boc("no account provided"))?;
            execute_message_tvm_only(&context, account, message, options)
        }
    }?;
    output.convert_to_result(&context, abi.as_ref())
}

// Full

fn execute_message_full(
    _context: &Arc<ClientContext>,
    account: Option<ton_types::Cell>,
    msg: ton_block::Message,
    options: ResolvedExecutionOptions,
) -> ClientResult<ExecutionOutput> {
    // if no account provided use AccountNone
    let account = match account {
        Some(account) => account,
        None => ton_block::Account::AccountNone.write_to_new_cell().unwrap().into()
    };
    let account_copy = account.clone();
    let msg_address = msg.dst()
        .ok_or(Error::invalid_message_type())?;
    let contract_info = move || {
        let account: ton_block::Account = deserialize_object_from_cell(account_copy.clone(), "account")?;
        match account.stuff() {
            Some(stuff) => {
                let balance = stuff.storage.balance.grams.value().to_u64().unwrap_or_default();
                Ok((stuff.addr.clone(), balance))
            },
            None => {
                Ok((msg_address.clone(), 0))
            }
        }
    };

    let output = call_executor(
        account,
        msg,
        options,
        &contract_info,
    )?;

    if let Some(transaction) = &output.transaction {
        check_transaction_status(transaction, false, &contract_info)?;
    }

    Ok(output)
}

fn call_executor(
    mut account: ton_types::Cell,
    msg: ton_block::Message,
    options: ResolvedExecutionOptions,
    contract_info: impl FnOnce() -> ClientResult<(ton_block::MsgAddressInt, u64)>,
) -> ClientResult<ExecutionOutput> {
    let executor = OrdinaryTransactionExecutor::new(options.blockchain_config);
    let transaction = executor
        .execute(
            Some(&msg),
            &mut account,
            options.block_time,
            options.block_lt,
            Arc::new(AtomicU64::new(options.transaction_lt)),
            false,
        )
        .map_err(|err| {
            let err_message = err.to_string();
            match contract_info() {
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
            }
        })?;
    Ok(ExecutionOutput::with_transaction(transaction, account))
}

// TVM Only

fn execute_message_tvm_only(
    _context: &Arc<ClientContext>,
    account: ton_types::Cell,
    message: ton_block::Message,
    options: ResolvedExecutionOptions,
) -> ClientResult<ExecutionOutput> {
    let account: ton_block::Account = deserialize_object_from_cell(account, "account")?;

    let stuff = match account {
        ton_block::Account::AccountNone => Err(Error::invalid_account_boc("Acount is None")),
        ton_block::Account::Account(stuff) => Ok(stuff)
    }?;

    let (messages, stuff) = call_tvm_msg(
        stuff,
        options,
        &message,
    )?;

    let account = serialize_account_stuff(stuff)?;

    Ok(ExecutionOutput::with_messages(messages, account))
}

pub(crate) fn call_tvm(
    mut account: ton_block::AccountStuff,
    options: ResolvedExecutionOptions,
    stack: Stack,
) -> ClientResult<(ton_vm::executor::Engine, ton_block::AccountStuff)> {
    let mut state = match &mut account.storage.state {
        ton_block::AccountState::AccountActive(state) => Ok(state),
        _ => Err(Error::invalid_account_boc("Account is not active"))
    }?;

    let mut ctrls = SaveList::new();
    ctrls.put(4, &mut StackItem::Cell(state.data.clone().unwrap_or_default()))
        .map_err(|err| Error::internal_error(format!("can not put data to registers: {}", err)))?;

    let sci = build_contract_info(
        options.blockchain_config.raw_config(),
        &account.addr,
        &account.storage.balance,
        options.block_time,
        options.block_lt, options.transaction_lt);
    ctrls.put(7, &mut sci.into_temp_data())
        .map_err(|err| Error::internal_error(format!("can not put SCI to registers: {}", err)))?;

    let gas_limit = 1_000_000_000;
    let gas = Gas::new(gas_limit, 0, gas_limit, 10);

    let code = state.code
        .clone()
        .ok_or(Error::invalid_account_boc("Account has no code"))?;
    let mut engine = ton_vm::executor::Engine::new().setup(
        SliceData::from(code),
        Some(ctrls),
        Some(stack),
        Some(gas),
    );

    let result = engine.execute();

    match result {
        Err(err) => {
            let exception = ton_vm::error::tvm_exception(err)
                .map_err(|err| Error::unknown_execution_error(err))?;
            let code = if let Some(code) = exception.custom_code() {
                code
            } else {
                !(exception.exception_code().unwrap_or(ton_types::ExceptionCode::UnknownError) as i32)
            };

            let exit_arg = super::stack::serialize_item(&exception.value)?;
            Err(Error::tvm_execution_failed(
                exception.to_string(), code, Some(exit_arg), &account.addr))
        }
        Ok(_) => {
            match engine.get_committed_state().get_root() {
                StackItem::Cell(cell) => state.data = Some(cell),
                _ => return Err(Error::internal_error("invalid commited state"))
            };
            Ok((engine, account))
        }
    }
}

fn call_tvm_msg(
    account: ton_block::AccountStuff,
    options: ResolvedExecutionOptions,
    msg: &Message
) -> ClientResult<(Vec<Message>, ton_block::AccountStuff)> {
    let msg_cell = msg.write_to_new_cell()
        .map_err(|err| Error::internal_error(format!("can not serialize message: {}", err)))?;
    let mut stack = Stack::new();
    let balance = account.storage.balance.grams.value();
    stack
        .push(ton_vm::int!(balance))                            // gram balance of contract
        .push(ton_vm::int!(0))                                  // gram balance of msg
        .push(StackItem::Cell(msg_cell.into()))                        // message
        .push(StackItem::Slice(msg.body().unwrap_or_default())) // message body
        .push(ton_vm::int!(-1));                                            // external inbound message flag

    let (engine, account) = call_tvm(account, options, stack)?;
    let mut slice = SliceData::from(
        engine.get_actions().as_cell()
            .map_err(|err| Error::internal_error(format!("can not get actions: {}", err)))?
    );
    let mut msgs = vec![];
    while slice.remaining_references() != 0 {
        let next = slice.checked_drain_reference().unwrap().into();
        let magic = slice.get_next_u32();
        if magic.is_ok() && magic.unwrap() == 0x0ec3c86d && slice.remaining_references() == 1 {
            let message = Message::construct_from(
                &mut slice.checked_drain_reference().unwrap().into()
            )
            .map_err(|err| 
                Error::internal_error(format!("contract produced invalid message: {}", err))
            )?;

            msgs.push(message);
        }
        slice = next;
    }
    msgs.reverse();
    Ok((msgs, account))
}

fn build_contract_info(
    config_params: &ton_block::ConfigParams,
    address: &ton_block::MsgAddressInt,
    balance: &ton_block::CurrencyCollection,
    block_unixtime: u32,
    block_lt: u64,
    tr_lt: u64
) -> ton_vm::SmartContractInfo {
    let mut info = ton_vm::SmartContractInfo::with_myself(
        address.serialize().unwrap_or_default().into());
    *info.block_lt_mut() = block_lt;
    *info.trans_lt_mut() = tr_lt;
    *info.unix_time_mut() = block_unixtime;
    *info.balance_remaining_grams_mut() = balance.grams.0;
    *info.balance_remaining_other_mut() = balance.other_as_hashmap();

    if let Some(data) = config_params.config_params.data() {
        info.set_config_params(data.clone());
    }
    info
}

pub(crate) fn serialize_account_stuff(stuff: ton_block::AccountStuff) -> ClientResult<Cell> {
    Ok(ton_block::Account::Account(stuff)
        .write_to_new_cell()
        .map_err(|err| Error::internal_error(format!("can not serialize account: {}", err)))?
        .into())
}
