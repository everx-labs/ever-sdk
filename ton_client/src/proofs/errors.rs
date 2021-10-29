use std::fmt::Display;

use crate::error::ClientError;

#[derive(ApiType)]
pub enum ErrorCode {
    InvalidData = 901,
    ProofCheckFailed = 902,
    InternalError = 903,
    DataDiffersFromProven = 904,
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

    pub fn proof_check_failed(err: impl Display) -> ClientError {
        error(
            ErrorCode::ProofCheckFailed,
            format!("Proof check failed: {}", err),
        )
    }

    pub fn data_differs_from_proven(err: impl Display) -> ClientError {
        error(
            ErrorCode::DataDiffersFromProven,
            format!("Data differs from the proven: {}", err),
        )
    }

    pub fn internal_error(err: impl Display) -> ClientError {
        error(
            ErrorCode::InternalError,
            format!("Internal error during proof checking: {}", err),
        )
    }
}
