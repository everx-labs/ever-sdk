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
    deserialize_cell_from_base64, deserialize_object_from_base64, serialize_cell_to_base64,
    serialize_object_to_base64,
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
    pub account: String,
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
    pub account: Option<Value>,
}

use crate::abi::MessageSource;
use crate::boc::{parse_account, parse_message, parse_transaction, ParamsOfParse, ResultOfParse};
use crate::processing::parsing::decode_output;
use crate::tvm::check_transaction::check_transaction_status;
use std::sync::Arc;
use ton_block;
use ton_block::{Message, Serializable, Transaction};
use ton_sdk::call_tvm;
use ton_types::{Cell, SliceData};

struct ExecutionOutput {
    transaction: Option<Transaction>,
    messages: Option<Vec<Message>>,
    account: Option<Cell>,
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
            account: Some(account),
        }
    }

    fn with_messages(messages: Vec<Message>, account: Option<Cell>) -> Self {
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

    fn convert_account(&self, context: &Arc<ClientContext>) -> ClientResult<Option<Value>> {
        self.account
            .as_ref()
            .map(|x| {
                parse_account(
                    context.clone(),
                    ParamsOfParse {
                        boc: serialize_cell_to_base64(x, "account")?,
                    },
                )
                .map(|parsed| parsed.parsed)
            })
            .transpose()
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
    let (_, account) = deserialize_cell_from_base64(&params.account, "account")?;
    let (message, abi) = params.message.encode(&context).await?;
    let message = deserialize_object_from_base64::<ton_block::Message>(&message, "message")?.object;
    let output = match params.mode {
        ExecutionMode::Full => {
            execute_message_full(&context, account, message, params.execution_options)
        }
        ExecutionMode::TvmOnly => {
            execute_message_tvm_only(&context, account, message, params.execution_options)
        }
    }?;
    output.convert_to_result(&context, abi.as_ref())
}

// Full

fn execute_message_full(
    context: &Arc<ClientContext>,
    account: ton_types::Cell,
    msg: ton_block::Message,
    options: Option<ExecutionOptions>,
) -> ClientResult<ExecutionOutput> {
    let options = options.unwrap_or_default();

    let account_copy = account.clone();
    let contract_info = move || {
        let account =
            ton_sdk::Contract::from_cells(account_copy.clone().into()).map_err(|err| {
                crate::boc::Error::invalid_boc(format!("Can not read account data: {}", err))
            })?;

        Ok((account.address(), account.balance))
    };

    let config = if let Some(config) = options.blockchain_config {
        blockchain_config_from_base64(&config)?
    } else {
        Default::default()
    };
    let output = call_executor(
        account,
        msg,
        config,
        options
            .block_time
            .unwrap_or_else(|| (context.env.now_ms() / 1000) as u32),
        options.block_lt,
        options.transaction_lt,
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
    config: BlockchainConfig,
    timestamp: u32,
    block_lt: Option<u64>,
    transaction_lt: Option<u64>,
    contract_info: impl FnOnce() -> ClientResult<(ton_block::MsgAddressInt, u64)>,
) -> ClientResult<ExecutionOutput> {
    let block_lt = block_lt.unwrap_or(transaction_lt.unwrap_or(1_000_001) - 1);
    let lt = Arc::new(std::sync::atomic::AtomicU64::new(
        transaction_lt.unwrap_or(block_lt + 1),
    ));
    let executor = OrdinaryTransactionExecutor::new(config);
    let transaction = executor
        .execute(
            Some(&msg),
            &mut account,
            timestamp,
            block_lt,
            lt.clone(),
            false,
        )
        .map_err(|err| {
            let err_message = err.to_string();
            match contract_info() {
                Ok((address, balance)) => match &err.downcast_ref::<ExecutorError>() {
                    Some(ExecutorError::NoAcceptError(code)) => {
                        Error::tvm_execution_failed(err_message, *code, &address)
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
    context: &Arc<ClientContext>,
    account: ton_types::Cell,
    message: ton_block::Message,
    _options: Option<ExecutionOptions>,
) -> ClientResult<ExecutionOutput> {
    let contract = ton_sdk::Contract::from_cells(SliceData::from(account))
        .map_err(|err| Error::invalid_account_boc(err))?;
    let code = contract
        .get_code()
        .ok_or(Error::invalid_account_boc("Account has no code"))?;

    let (messages, _) = call_tvm(
        contract.balance,
        contract
            .balance_other_as_hashmape()
            .map_err(|err| Error::invalid_account_boc(err))?,
        &contract.id,
        None,
        (context.env.now_ms() / 1000) as u32,
        code,
        contract.get_data(),
        &message,
    )
    .map_err(|err| Error::unknown_execution_error(err))?;

    Ok(ExecutionOutput::with_messages(messages, None))
}
