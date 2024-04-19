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
 *
 */

use super::types::ResolvedExecutionOptions;
use crate::{error::ClientResult, encoding::slice_from_cell};
use crate::tvm::Error;
use ever_block::{
    Account, CommonMsgInfo, ConfigParams, CurrencyCollection, Deserializable,
    Message, MsgAddressInt, OutAction, OutActions, Serializable,
};
use ever_block::{Cell, HashmapType, SliceData, UInt256};
use ever_vm::{
    executor::{gas::gas_state::Gas, Engine},
    stack::{integer::IntegerData, savelist::SaveList, Stack, StackItem},
};

pub(crate) fn call_tvm(
    account: &mut Account,
    options: ResolvedExecutionOptions,
    stack: Stack,
) -> ClientResult<Engine> {
    let code = account.get_code().unwrap_or_default();
    let data = account
        .get_data()
        .ok_or_else(|| Error::invalid_account_boc("Account has no code"))?;
    let addr = account
        .get_addr()
        .ok_or_else(|| Error::invalid_account_boc("Account has no address"))?;
    let balance = account
        .balance()
        .ok_or_else(|| Error::invalid_account_boc("Account has no balance"))?;

    let mut ctrls = SaveList::new();
    ctrls
        .put(4, &mut StackItem::Cell(data))
        .map_err(|err| Error::internal_error(format!("can not put data to registers: {}", err)))?;

    let mut sci = build_contract_info(
        options.blockchain_config.raw_config(),
        addr,
        balance,
        options.block_time,
        options.block_lt,
        options.transaction_lt,
        code.clone(),
        account.init_code_hash(),
    );
    sci.capabilities = options.blockchain_config.capabilites();
    ctrls
        .put(7, &mut sci.into_temp_data_item())
        .map_err(|err| Error::internal_error(format!("can not put SCI to registers: {}", err)))?;

    let gas_limit = 1_000_000_000;
    let gas = Gas::new(gas_limit, 0, gas_limit, 10);

    let mut engine = Engine::with_capabilities(
        options.blockchain_config.capabilites()
    ).setup(
        slice_from_cell(code)?,
        Some(ctrls),
        Some(stack),
        Some(gas),
    );

    engine.set_signature_id(options.signature_id);
    engine.modify_behavior(options.behavior_modifiers);

    match engine.execute() {
        Err(err) => {
            let exception = ever_vm::error::tvm_exception(err)
                .map_err(|err| Error::unknown_execution_error(err))?;
            let code = if let Some(code) = exception.custom_code() {
                code
            } else {
                !(exception
                    .exception_code()
                    .unwrap_or(ever_block::ExceptionCode::UnknownError) as i32)
            };

            let exit_arg = super::stack::serialize_item(&exception.value)?;
            Err(Error::tvm_execution_failed(
                exception.to_string(),
                code,
                Some(exit_arg),
                addr,
                None,
                true,
            ))
        }
        Ok(_) => match engine.get_committed_state().get_root() {
            StackItem::Cell(data) => {
                account.set_data(data.clone());
                Ok(engine)
            }
            _ => Err(Error::internal_error("invalid committed state")),
        },
    }
}

pub(crate) fn call_tvm_msg(
    account: &mut Account,
    options: ResolvedExecutionOptions,
    msg: &Message,
) -> ClientResult<Vec<Message>> {
    let msg_cell = msg
        .serialize()
        .map_err(|err| Error::internal_error(format!("can not serialize message: {}", err)))?;

    let mut stack = Stack::new();
    let balance = account.balance().map_or(0, |cc| cc.grams.as_u128());
    let function_selector = match msg.header() {
        CommonMsgInfo::IntMsgInfo(_) => ever_vm::int!(0),
        CommonMsgInfo::ExtInMsgInfo(_) => ever_vm::int!(-1),
        CommonMsgInfo::ExtOutMsgInfo(_) => return Err(Error::invalid_message_type()),
    };
    stack
        .push(ever_vm::int!(balance)) // token balance of contract
        .push(ever_vm::int!(0)) // token balance of msg
        .push(StackItem::Cell(msg_cell.into())) // message
        .push(StackItem::Slice(msg.body().unwrap_or_default())) // message body
        .push(function_selector); // function selector

    let engine = call_tvm(account, options, stack)?;

    // process out actions to get out messages
    let actions_cell = engine
        .get_actions()
        .as_cell()
        .map_err(|err| Error::internal_error(format!("can not get actions: {}", err)))?
        .clone();
    let mut actions = OutActions::construct_from_cell(actions_cell)
        .map_err(|err| Error::internal_error(format!("can not parse actions: {}", err)))?;

    let mut msgs = vec![];
    for (_, action) in actions.iter_mut().enumerate() {
        match std::mem::replace(action, OutAction::None) {
            OutAction::SendMsg { out_msg, .. } => {
                msgs.push(out_msg);
            }
            _ => {}
        }
    }

    msgs.reverse();
    Ok(msgs)
}

fn build_contract_info(
    config_params: &ConfigParams,
    address: &MsgAddressInt,
    balance: &CurrencyCollection,
    block_unixtime: u32,
    block_lt: u64,
    tr_lt: u64,
    code: Cell,
    init_code_hash: Option<&UInt256>,
) -> ever_vm::SmartContractInfo {
    let mut info =
        ever_vm::SmartContractInfo::with_myself(
            address.serialize().and_then(|cell| SliceData::load_cell(cell)).unwrap_or_default()
        );
    info.block_lt = block_lt;
    info.trans_lt = tr_lt;
    info.unix_time = block_unixtime;
    info.balance = balance.clone();
    if let Some(data) = config_params.config_params.data() {
        info.config_params = Some(data.clone());
    }
    if let Some(hash) = init_code_hash {
        info.set_init_code_hash(hash.clone());
    }
    info.set_mycode(code);
    info
}
