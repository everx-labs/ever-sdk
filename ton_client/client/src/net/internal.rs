use crate::error::ApiResult;
use crate::client::ClientContext;
use crate::net::types::{CallbackParams, MessageProcessingEvent};

pub(crate) fn get_message_id(message: &TvmMessage) -> ApiResult<Vec<u8>> {
    let cells = message.write_to_new_cell()?.into();
    Ok(cells.repr_hash().as_slice()[..].into())
}

pub(crate) fn emit_event(
    context: &Arc<ClientContext>,
    callback: &Option<CallbackParams>,
    event: fn() -> MessageProcessingEvent,
) {
    if let Some(callback) = callback {
        let _ = context.send_callback_result(callback.id.clone(), event());
    }
}

