use crate::error::ApiError;
use std::fmt::Display;

pub enum Code {
    InvalidHex = 10,
    InvalidBase64 = 11,
    InvalidAddress = 12,
}
pub struct Error;

fn error(code: Code, message: String) -> ApiError {
    ApiError::with_code_message(code as isize, message)
}

impl Error {
    pub fn invalid_hex<E: Display>(s: &String, err: E) -> ApiError {
        error(
            Code::InvalidHex,
            format!("Invalid hex string: {}\r\nhex: [{}]", err, s),
        )
    }

    pub fn invalid_base64<E: Display>(s: &String, err: E) -> ApiError {
        error(
            Code::InvalidBase64,
            format!("Invalid base64 string: {}\r\nbase64: [{}]", err, s),
        )
    }

    pub fn invalid_address<E: Display>(err: E, address: &str) -> ApiError {
        error(
            Code::InvalidAddress,
            format!("Invalid address [{}]: {}", err, address),
        )
    }


}
