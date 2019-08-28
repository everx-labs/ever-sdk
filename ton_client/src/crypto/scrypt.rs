extern crate scrypt;

use types::{ApiResult, ApiError};

pub fn scrypt(password: &[u8], salt: &[u8], log_n: u8, r: u32, p: u32, dk_len: usize) -> ApiResult<Vec<u8>> {
    let mut result = Vec::new();
    result.resize(dk_len, 0);
    let params = scrypt::ScryptParams::new(log_n, r, p)
        .map_err(|err|ApiError::crypto_scrypt_failed(err))?;
    scrypt::scrypt(password, salt, &params, & mut result)
        .map_err(|err|ApiError::crypto_scrypt_failed(err))?;
    Ok(result)
}


