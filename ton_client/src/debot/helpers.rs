use crate::boc::internal::serialize_object_to_base64;
use crate::encoding::account_decode;
use crate::error::ClientResult;
use ever_block::{InternalMessageHeader, Message};
use ever_block::SliceData;

pub(super) fn build_internal_message(
    src: &String,
    dst: &String,
    body: SliceData,
) -> ClientResult<String> {
    let src_addr = account_decode(src)?;
    let dst_addr = account_decode(dst)?;
    let mut msg = Message::with_int_header(InternalMessageHeader::with_addresses(
        src_addr,
        dst_addr,
        Default::default(),
    ));
    msg.set_body(body);
    serialize_object_to_base64(&msg, "message")
}
