use std::fmt::Display;

use crate::error::ClientError;

#[derive(ApiType)]
pub enum ErrorCode {
    InvalidData = 1,
    UnableToResolveZeroStateRootHash = 2,
    UnableToResolveTrustedKeyBlock = 3,
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

    pub fn unable_to_resolve_zerostate_root_hash(err: impl Display) -> ClientError {
        error(
            ErrorCode::UnableToResolveZeroStateRootHash,
            format!("Unable to resolve zerostate's root hash: {}", err),
        )
    }

    pub fn unable_to_resolve_trusted_key_block(zerostate_root_hash: &str) -> ClientError {
        error(
            ErrorCode::UnableToResolveTrustedKeyBlock,
            format!(
                "Unable to resolve trusted key-block for network with zerostate root_hash: `{}`",
                zerostate_root_hash,
            ),
        )
    }
}
