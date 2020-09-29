use crate::error::ApiResult;
use ton_block::Serializable;
use crate::processing::Error;

pub(crate) fn get_message_id(message: &ton_block::Message) -> ApiResult<String> {
    let cells: ton_types::Cell = message
        .write_to_new_cell()
        .map_err(|err| Error::can_not_build_message_cell(err))?
        .into();
    let id: Vec<u8> = cells.repr_hash().as_slice()[..].into();
    Ok(hex::encode(&id))
}

