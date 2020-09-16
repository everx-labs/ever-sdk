use crate::error::ApiError;
use std::fmt::Display;
const CLIENT: isize = ApiError::CLIENT; // 0

pub enum ErrorCode {
    NotImplemented = CLIENT + 1,
    InvalidHex = CLIENT + 2,
    InvalidBase64 = CLIENT + 3,
    InvalidAddress = CLIENT + 4,
}
pub struct Error;

fn error(code: ErrorCode, message: String) -> ApiError {
    ApiError::with_code_message(code as isize, message)
}

impl Error {
    pub fn not_implemented(message: &str) -> ApiError {
        error(ErrorCode::NotImplemented, message.into())
    }

    pub fn invalid_hex<E: Display>(s: &String, err: E) -> ApiError {
        error(
            ErrorCode::InvalidHex,
            format!("Invalid hex string: {}\r\nhex: [{}]", err, s),
        )
    }

    pub fn invalid_base64<E: Display>(s: &String, err: E) -> ApiError {
        error(
            ErrorCode::InvalidBase64,
            format!("Invalid base64 string: {}\r\nbase64: [{}]", err, s),
        )
    }

    pub fn invalid_address<E: Display>(err: E, address: &str) -> ApiError {
        error(
            ErrorCode::InvalidAddress,
            format!("Invalid address [{}]: {}", err, address),
        )
    }
}
