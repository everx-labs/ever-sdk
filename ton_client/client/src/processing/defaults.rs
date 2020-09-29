pub const DEFAULT_NETWORK_RETRIES_LIMIT: i8 = -1;
pub const DEFAULT_NETWORK_RETRIES_TIMEOUT: u32 = 1000;
pub const DEFAULT_EXPIRATION_RETRIES_LIMIT: i8 = 20;
pub const DEFAULT_EXPIRATION_RETRIES_TIMEOUT: u32 = 1000;

pub(crate) fn can_retry_more(retries: &mut i8, limit: i8) -> bool {
    *retries = *retries.checked_add(1).unwrap_or(*retries);
    limit < 0 || *retries <= limit
}
