use std::sync::Arc;
use tvm::executor::Engine;
use tvm::block::{
    BlockResult,
    Message,
    Serializable,
    Deserializable,
};
use tvm::stack::{CellData, IntegerData, SaveList, SliceData, Stack, StackItem};

#[cfg(test)]
#[path = "tests/test_local_tvm.rs"]
mod tests;

#[allow(dead_code)]
pub fn local_contract_call(code: SliceData, data: Arc<CellData>, msg: &Message)
-> BlockResult<Vec<Message>> {
    let msg_cell = msg.write_to_new_cell()?.into();
    let mut stack = Stack::new();
    stack
        .push(int!(0))                                          // gram balance of contract
        .push(int!(0))                                          // gram balance of msg
        .push(StackItem::Cell(msg_cell))                        // message
        .push(StackItem::Slice(msg.body().unwrap_or_default())) // message body
        .push(int!(0));                                         // external inbound message flag
    let mut ctrls = SaveList::new();
    ctrls.put(4, &mut StackItem::Cell(data)).unwrap();
    let mut engine = Engine::new().setup(code, Some(ctrls), Some(stack), None);
    let _result = engine.execute()?;
    let mut slice = SliceData::from(engine.get_actions().as_cell()?.clone());

    let mut msgs = vec![];
    while &slice.get_bytestring(0) == &[0x0e, 0xc3, 0xc8, 0x6d, 0x00] && slice.remaining_references() == 2 {
        let next = slice.drain_reference().into();
        msgs.push(Message::construct_from::<Message>(&mut slice.drain_reference().into())?);
        slice = next;
    }
    msgs.reverse();
    Ok(msgs)
}
