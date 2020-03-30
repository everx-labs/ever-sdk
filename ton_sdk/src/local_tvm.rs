/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::error::{SdkError, SdkErrorKind, SdkResult};
use std::sync::Arc;
use ton_vm::executor::Engine;
use ton_block::{
    Message,
    Serializable,
    Deserializable,
    MsgAddressInt,
};
use ton_types::{Cell, SliceData, HashmapE};
use ton_vm::stack::{integer::IntegerData, savelist::SaveList, Stack, StackItem};
use ton_vm::SmartContractInfo;
use ton_vm::executor::gas::gas_state::Gas;
#[cfg(feature = "fee_calculation")]
use ton_executor::{BlockchainConfig, TransactionExecutor, OrdinaryTransactionExecutor};

pub(crate) fn call_tvm(
    balance: u128,
    balance_other: HashmapE,
    address: &MsgAddressInt,
    config_params: Option<Cell>,
    timestamp: u32,
    code:Cell,
    data: Option<Cell>,
    msg: &Message)
-> SdkResult<(Vec<Message>, i64)> {
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
        .map_err(|err| SdkError::from(SdkErrorKind::InternalError {
            msg: format!("Cannot put data to register: {}", err)
        }))?;

    let mut sci = SmartContractInfo::with_myself(address.write_to_new_cell()?.into());
    *sci.unix_time_mut() = timestamp;
    *sci.balance_remaining_grams_mut() = balance;
    *sci.balance_remaining_other_mut() = balance_other;
    if let Some(params) = config_params {
        sci.set_config_params(params);
    }

    ctrls.put(7, &mut sci.into_temp_data())
        .map_err(|err| SdkError::from(SdkErrorKind::InternalError {
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
            msgs.push(Message::construct_from::<Message>(&mut slice.checked_drain_reference()?.into())?);
        }
        slice = next;
    }
    msgs.reverse();
    Ok((msgs, engine.gas_used()))
}

#[cfg(feature = "fee_calculation")]
pub mod executor {
    
use super::*;
use num_traits::cast::ToPrimitive;
use ton_types::types::UInt256;
use ton_block::types::Grams;
use ton_block::{
    Account,
    AccStatusChange,
    Message,
    ShardAccount,
    TransactionDescr,
    TrComputePhase
};

#[derive(Default, Debug)]
pub struct TransactionFees {
    pub in_msg_fwd_fee: u64,
    pub storage_fee: u64,
    pub gas_fee: u64,
    pub out_msgs_fwd_fee: u64,
    pub total_account_fees: u64,
    pub total_output: u64,
}

fn grams_to_u64(grams: &ton_block::types::Grams) -> SdkResult<u64> {
    grams.0.to_u64()
        .ok_or(SdkErrorKind::LocalCallError { msg: "Cannot convert rams value".to_owned() }.into())
}

pub(crate) fn call_executor(account: Account, msg: Message, config: &BlockchainConfig, timestamp: u32)
    -> SdkResult<(Vec<Message>, TransactionFees)>
{
    let shard_acc = ShardAccount::with_params(account, UInt256::from([0;32]), 0).unwrap();

    let mut executor = OrdinaryTransactionExecutor::new();
    let transaction = executor.execute(
        config,
        msg,
        &mut Some(shard_acc),
        timestamp,
        1,
        false)?;

    let mut fees = TransactionFees::default();

    if let TransactionDescr::Ordinary(descr) = transaction.read_description()? {
        let is_aborted = descr.aborted;

        if let Some(storage_phase) = descr.storage_ph {
            if storage_phase.status_change != AccStatusChange::Unchanged {
                bail!(SdkErrorKind::LocalCallError {
                    msg: format!("Storage phase failed. Status change: {:?}", storage_phase.status_change)
                });
            }
            fees.storage_fee = grams_to_u64(&storage_phase.storage_fees_collected)?;
        } else {
            if is_aborted {
                bail!(SdkErrorKind::LocalCallError { msg: "No storage phase".to_owned() } );
            }
        }

        fees.gas_fee = match descr.compute_ph {
            TrComputePhase::Vm(phase) => { 
                if !phase.success {
                    bail!(SdkErrorKind::LocalCallError {
                        msg: format!("Compute phase failed. Exit code: {}", phase.exit_code) } )
                }
                grams_to_u64(&phase.gas_fees)?
            },
            TrComputePhase::Skipped(skipped) => bail!(SdkErrorKind::LocalCallError {
                msg: format!("Compute phase skipped. Reason: {:?}", skipped.reason) } )
        };

        let action_phase = descr.action
            .ok_or(SdkErrorKind::LocalCallError { msg: "No action phase".to_owned() } )?;
        if !action_phase.success {
            bail!(SdkErrorKind::LocalCallError {
                msg: format!("Action phase failed. Result: {:?}", action_phase.result_code) } );
        }
        fees.out_msgs_fwd_fee = grams_to_u64(&action_phase.total_fwd_fees.unwrap_or_default())?;

        let tr_total_fees = grams_to_u64(&transaction.total_fees().grams)?;
        let total_action_fees = grams_to_u64(&action_phase.total_action_fees.unwrap_or_default())?;

        // `transaction.total_fees` is calculated as
        // `transaction.total_fees = inbound_fwd_fees + storage_fees + gas_fees + total_action_fees`
        // but this total_fees is fees collected the validators, not the all fees taken from account
        // because total_action_fees contains only part of all forward fees
        // to get all fees paid by account we need exchange `total_action_fees part` to `out_msgs_fwd_fee`
        fees.total_account_fees = tr_total_fees - total_action_fees + fees.out_msgs_fwd_fee;
        // inbound_fwd_fees is not represented in transaction fields so need to calculate it
        fees.in_msg_fwd_fee = fees.total_account_fees - fees.storage_fee - fees.gas_fee - fees.out_msgs_fwd_fee;
    } else {
        return Err(SdkErrorKind::LocalCallError { msg: "Invalid transaction type".to_owned() }.into());
    }

    let mut messages = vec![];
    let mut total_output = Grams::zero();
    transaction.iterate_out_msgs(&mut |msg| { 
        if let Some(value) = msg.get_value() {
            total_output.0 += &value.grams.0;
        }
        messages.push(msg);
        Ok(true) 
    })?;

    fees.total_output = grams_to_u64(&total_output)?;

    Ok((messages, fees))
}

}


#[cfg(test)]
#[path = "tests/test_local_tvm.rs"]
mod tests;
