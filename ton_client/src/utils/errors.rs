use std::fmt::Display;

use crate::error::ClientError;

#[derive(ApiType)]
pub enum ErrorCode {
    CompressionError = 701,
}

pub struct Error;

fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as u32, message)
}

impl Error {
    pub fn compression_error<E: Display>(err: E) -> ClientError {
        error(ErrorCode::CompressionError, format!("Compression error: {}", err))
    }

    pub fn decompression_error<E: Display>(err: E) -> ClientError {
        error(ErrorCode::CompressionError, format!("Decompression error: {}", err))
    }
}
