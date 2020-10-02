use crate::client::ClientContext;
use crate::error::ApiResult;
use crate::processing::{
    Error, DEFAULT_EXPIRATION_RETRIES_LIMIT, DEFAULT_EXPIRATION_RETRIES_TIMEOUT,
    DEFAULT_NETWORK_RETRIES_LIMIT, DEFAULT_NETWORK_RETRIES_TIMEOUT,
};
use std::sync::Arc;
use ton_block::Serializable;

pub(crate) fn get_message_id(message: &ton_block::Message) -> ApiResult<String> {
    let cells: ton_types::Cell = message
        .write_to_new_cell()
        .map_err(|err| Error::can_not_build_message_cell(err))?
        .into();
    let id: Vec<u8> = cells.repr_hash().as_slice()[..].into();
    Ok(hex::encode(&id))
}

/// Increments `retries` and returns `true` if `retries` isn't reach `limit`.
pub(crate) fn can_retry_more(retries: &mut i8, limit: i8) -> bool {
    *retries = retries.checked_add(1).unwrap_or(*retries);
    limit < 0 || *retries <= limit
}

pub fn can_retry_network_error(context: &Arc<ClientContext>, retries: &mut i8) -> bool {
    can_retry_more(
        retries,
        resolve(
            context.config.network.as_ref(),
            |_| None,
            DEFAULT_NETWORK_RETRIES_LIMIT,
        ),
    )
}

pub fn resolve_network_retries_timeout(context: &Arc<ClientContext>) -> u32 {
    resolve(
        context.config.network.as_ref(),
        |_| None,
        DEFAULT_NETWORK_RETRIES_TIMEOUT,
    )
}

pub(crate) fn can_retry_expired_message(context: &Arc<ClientContext>, retries: &mut i8) -> bool {
    can_retry_more(
        retries,
        resolve(
            context.config.network.as_ref(),
            |x| Some(x.message_retries_count() as i8),
            DEFAULT_EXPIRATION_RETRIES_LIMIT,
        ),
    )
}

pub fn resolve_expiration_retries_timeout(context: &Arc<ClientContext>) -> u32 {
    resolve(
        context.config.network.as_ref(),
        |x| Some(x.message_processing_timeout()),
        DEFAULT_EXPIRATION_RETRIES_TIMEOUT,
    )
}

fn resolve<C, R>(config: Option<&C>, resolve_cfg: fn(cfg: &C) -> Option<R>, def: R) -> R {
    let cfg = config.map_or(None, |x| resolve_cfg(x));
    cfg.unwrap_or(def)
}
