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

use std::sync::Arc;
use ton_block;
use ton_block::{Deserializable, Message, Serializable};
use ton_vm::stack::{integer::IntegerData, savelist::SaveList, Stack, StackItem};
use ton_types::SliceData;
use ton_types::dictionary::HashmapType;
use ton_vm::executor::gas::gas_state::Gas;
use crate::error::ClientResult;
use crate::tvm::Error;
use super::types::ResolvedExecutionOptions;

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

pub(crate) fn call_tvm_msg(
    account: ton_block::AccountStuff,
    options: ResolvedExecutionOptions,
    msg: &Message
) -> ClientResult<(Vec<Message>, ton_block::AccountStuff)> {
    let msg_cell = msg.write_to_new_cell()
        .map_err(|err| Error::internal_error(format!("can not serialize message: {}", err)))?;

    let mut stack = Stack::new();
    let balance = account.storage.balance.grams.value();
    let function_selector = match msg.header() {
        ton_block::CommonMsgInfo::IntMsgInfo(_) => ton_vm::int!(0),
        ton_block::CommonMsgInfo::ExtInMsgInfo(_) => ton_vm::int!(-1),
        ton_block::CommonMsgInfo::ExtOutMsgInfo(_) => 
            return Err(Error::invalid_message_type())
    };
    stack
        .push(ton_vm::int!(balance))                            // token balance of contract
        .push(ton_vm::int!(0))                                  // token balance of msg
        .push(StackItem::Cell(msg_cell.into()))                 // message
        .push(StackItem::Slice(msg.body().unwrap_or_default())) // message body
        .push(function_selector);                                           // function selector

    let (engine, account) = call_tvm(account, options, stack)?;

    // process out actions to get out messages
    let actions_cell = engine
        .get_actions()
        .as_cell()
        .map_err(|err| Error::internal_error(format!("can not get actions: {}", err)))?
        .clone();
    let mut actions = ton_block::OutActions::construct_from_cell(actions_cell)
        .map_err(|err| Error::internal_error(format!("can not parse actions: {}", err)))?;

    let mut msgs = vec![];
    for (_, action) in actions.iter_mut().enumerate() {
        match std::mem::replace(action, ton_block::OutAction::None) {
            ton_block::OutAction::SendMsg{ out_msg, .. } => {
                msgs.push(out_msg);
            },
            _ => {}
        }
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
