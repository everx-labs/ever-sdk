/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
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

use crate::error::*;
use std::sync::Arc;
use chrono::prelude::*;
use std::convert::TryFrom;
use ton_vm::executor::Engine;
use ton_block::{
    Message,
    Serializable,
    Deserializable,
};
use ton_block::error::*;
use ton_vm::stack::{CellData, IntegerData, SaveList, SliceData, Stack, StackItem};
use ton_vm::SmartContractInfo;

#[cfg(test)]
#[path = "tests/test_local_tvm.rs"]
mod tests;

#[allow(dead_code)]
pub fn local_contract_call(code: Arc<CellData>, data: Option<Arc<CellData>>, msg: &Message)
-> SdkResult<Vec<Message>> {
    let msg_cell = msg.write_to_new_cell()?.into();
    let mut stack = Stack::new();
    stack
        .push(int!(0))                                          // gram balance of contract
        .push(int!(0))                                          // gram balance of msg
        .push(StackItem::Cell(msg_cell))                        // message
        .push(StackItem::Slice(msg.body().unwrap_or_default())) // message body
        .push(int!(-1));                                        // external inbound message flag
    
    let mut ctrls = SaveList::new();
    ctrls.put(4, &mut StackItem::Cell(data.unwrap_or_default()))
        .map_err(|err| BlockError::from(BlockErrorKind::Other(
            format!("Cannot put data to register: {}", err))))?;

    let mut sci = SmartContractInfo::default();
    *sci.unix_time_mut() = <u32>::try_from(Utc::now().timestamp())
        .map_err(|_| BlockError::from(BlockErrorKind::Other(
            format!("Wrong time: {}", Utc::now().timestamp()))))?;
    ctrls.put(7, &mut sci.into_temp_data())
        .map_err(|err| BlockError::from(BlockErrorKind::Other(
            format!("Cannot put data to register: {}", err))))?;
    
    let mut engine = Engine::new().setup(SliceData::from(code), Some(ctrls), Some(stack), None);
    let _result = engine.execute()?;
    let mut slice = SliceData::from(engine.get_actions().as_cell()?.clone());

    let mut msgs = vec![];
    while slice.remaining_references() != 0 {
        let next = slice.checked_drain_reference()?.into();
        if Ok(0x0ec3c86d) == slice.get_next_u32() && slice.remaining_references() == 1 {
            msgs.push(Message::construct_from::<Message>(&mut slice.checked_drain_reference()?.into())?);
        }
        slice = next;
    }
    msgs.reverse();
    Ok(msgs)
}
