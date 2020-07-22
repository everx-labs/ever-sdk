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

use crate::error::SdkError;
use std::sync::Arc;
use ton_vm::executor::Engine;
use ton_block::{
    Message,
    Serializable,
    Deserializable,
    MsgAddressInt,
    Transaction
};
use ton_types::{error, Result, Cell, SliceData, HashmapE};
use ton_vm::stack::{integer::IntegerData, savelist::SaveList, Stack, StackItem};
use ton_vm::SmartContractInfo;
use ton_vm::executor::gas::gas_state::Gas;
use ton_executor::{BlockchainConfig, TransactionExecutor, OrdinaryTransactionExecutor};

pub(crate) fn call_tvm_stack(
    balance: u64,
    balance_other: HashmapE,
    address: &MsgAddressInt,
    config_params: Option<Cell>,
    timestamp: u32,
    code: Cell,
    data: Option<Cell>,
    stack: Stack,
) -> Result<Stack> {
    let mut ctrls = SaveList::new();
    ctrls.put(4, &mut StackItem::Cell(data.unwrap_or_default()))
        .map_err(|err| error!(SdkError::InternalError {
            msg: format!("Cannot put data to register: {}", err)
        }))?;

    let mut sci = SmartContractInfo::with_myself(address.write_to_new_cell()?.into());
    *sci.unix_time_mut() = timestamp;
    *sci.balance_remaining_grams_mut() = balance as u128;
    *sci.balance_remaining_other_mut() = balance_other;
    if let Some(params) = config_params {
        sci.set_config_params(params);
    }

    ctrls.put(7, &mut sci.into_temp_data())
        .map_err(|err| error!(SdkError::InternalError {
            msg: format!("Cannot put data to register: {}", err)
        }))?;

    let gas_limit = 1_000_000_000;
    let gas = Gas::new(gas_limit, 0, gas_limit, 10);

    let mut engine = Engine::new().setup(
        SliceData::from(code),
        Some(ctrls),
        Some(stack),
        Some(gas),
    );
    let _result = engine.execute()?;
    Ok(engine.stack().clone())
}

pub(crate) fn call_tvm(
    balance: u64,
    balance_other: HashmapE,
    address: &MsgAddressInt,
    config_params: Option<Cell>,
    timestamp: u32,
    code: Cell,
    data: Option<Cell>,
    msg: &Message)
    -> Result<(Vec<Message>, i64)> {
    let msg_cell = msg.write_to_new_cell()?.into();
    let mut stack = Stack::new();
    stack
        .push(int!(balance))                                    // gram balance of contract
        .push(int!(0))                                          // gram balance of msg
        .push(StackItem::Cell(msg_cell))                        // message
        .push(StackItem::Slice(msg.body().unwrap_or_default())) // message body
        .push(int!(-1));                                        // external inbound message flag

    let mut ctrls = SaveList::new();
    ctrls.put(4, &mut StackItem::Cell(data.unwrap_or_default()))
        .map_err(|err| error!(SdkError::InternalError {
            msg: format!("Cannot put data to register: {}", err)
        }))?;

    let mut sci = SmartContractInfo::with_myself(address.write_to_new_cell()?.into());
    *sci.unix_time_mut() = timestamp;
    *sci.balance_remaining_grams_mut() = balance as u128;
    *sci.balance_remaining_other_mut() = balance_other;
    if let Some(params) = config_params {
        sci.set_config_params(params);
    }

    ctrls.put(7, &mut sci.into_temp_data())
        .map_err(|err| error!(SdkError::InternalError {
            msg: format!("Cannot put data to register: {}", err)
        }))?;

    let gas_limit = 1_000_000_000;
    let gas = Gas::new(gas_limit, 0, gas_limit, 10);

    let mut engine = Engine::new().setup(SliceData::from(code), Some(ctrls), Some(stack), Some(gas));
    let _result = engine.execute()?;
    let mut slice = SliceData::from(engine.get_actions().as_cell()?.clone());

    let mut msgs = vec![];
    while slice.remaining_references() != 0 {
        let next = slice.checked_drain_reference()?.into();
        let magic = slice.get_next_u32();
        if magic.is_ok() && magic.unwrap() == 0x0ec3c86d && slice.remaining_references() == 1 {
            msgs.push(Message::construct_from(&mut slice.checked_drain_reference()?.into())?);
        }
        slice = next;
    }
    msgs.reverse();
    Ok((msgs, engine.gas_used()))
}

pub mod executor {
    use super::*;
    use ton_block::{
        Account,
        Message,
        Serializable,
    };
    use ton_executor::ExecutorError;
    use std::sync::atomic::AtomicU64;

    pub(crate) fn call_executor(
        account: Account,
        msg: Message,
        config: BlockchainConfig,
        timestamp: u32,
        block_lt: Option<u64>,
        transaction_lt: Option<u64>,
    ) -> Result<(Transaction, Cell)> {
        let mut acc_root = account.write_to_new_cell()?.into();

        let block_lt = block_lt.unwrap_or(transaction_lt.unwrap_or(1_000_001) - 1);
        let last_lt = Arc::new(AtomicU64::new(transaction_lt.unwrap_or(block_lt + 2) - 1));
        let executor = OrdinaryTransactionExecutor::new(config);
        let transaction = executor.execute(
            Some(&msg),
            &mut acc_root,
            timestamp,
            block_lt,
            last_lt.clone(),
            false)
            .map_err(|err| {
                match err.downcast_ref::<ExecutorError>() {
                    Some(ExecutorError::NoAcceptError(code)) => SdkError::ContractError(*code).into(),
                    Some(ExecutorError::NoFundsToImportMsg) => SdkError::NoFundsError.into(),
                    _ => err
                }
            })?;
        Ok((transaction, acc_root))
    }
}


#[cfg(test)]
#[path = "tests/test_local_tvm.rs"]
mod tests;
