use std::sync::Arc;
use chrono::prelude::*;
use std::convert::TryFrom;
use tvm::executor::Engine;
use tvm::block::{
    BlockResult,
    Message,
    Serializable,
    Deserializable,
};
use tvm::block::error::*;
use tvm::stack::{CellData, IntegerData, SaveList, SliceData, Stack, StackItem};
use tvm::SmartContractInfo;

#[cfg(test)]
#[path = "tests/test_local_tvm.rs"]
mod tests;

#[allow(dead_code)]
pub fn local_contract_call(code: Arc<CellData>, data: Option<Arc<CellData>>, msg: &Message)
-> BlockResult<Vec<Message>> {
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
            format!("Cannot put data to register: {}", err))))?;;
    
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
