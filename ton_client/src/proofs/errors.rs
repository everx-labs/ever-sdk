use std::fmt::Display;

use crate::error::ClientError;

#[derive(ApiType)]
pub enum ErrorCode {
    InvalidData = 1,
}

pub struct Error;

fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as u32, message)
}

impl Error {
    pub fn invalid_data(err: impl Display) -> ClientError {
        error(
            ErrorCode::InvalidData,
            format!("Invalid data: {}", err),
        )
    }
}
