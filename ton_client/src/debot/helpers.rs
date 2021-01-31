use crate::encoding::{account_decode};
use ton_block::{InternalMessageHeader, Message};
use crate::boc::internal::{deserialize_cell_from_base64, serialize_object_to_base64};
use ton_types::SliceData;
use crate::error::{ClientError, ClientResult};

pub(super) fn build_internal_message(src: &String, dst: &String, body: SliceData) -> ClientResult<String> {
    let src_addr = account_decode(src)?;
    let dst_addr = account_decode(dst)?;
    let mut msg = Message::with_int_header(
        InternalMessageHeader::with_addresses(src_addr, dst_addr, Default::default())
    );
    msg.set_body(body);
    serialize_object_to_base64(&msg, "message")
}