use crate::client::ClientContext;
use crate::error::ApiResult;
use crate::processing::types::{CallbackParams, ProcessingEvent};
use crate::processing::Error;
use std::sync::Arc;
use ton_block::Serializable;

pub(crate) fn get_message_id(message: &ton_block::Message) -> ApiResult<Vec<u8>> {
    let cells: ton_types::Cell = message
        .write_to_new_cell()
        .map_err(|err| Error::can_not_build_message_cell(err))?
        .into();
    Ok(cells.repr_hash().as_slice()[..].into())
}

