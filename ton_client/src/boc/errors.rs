/*
* Copyright 2018-2021 TON Labs LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::error::ClientError;
use std::fmt::Display;

#[derive(ApiType)]
pub enum ErrorCode {
    InvalidBoc = 201,
    SerializationError = 202,
    InappropriateBlock = 203,
    MissingSourceBoc = 204,
    InsufficientCacheSize = 205,
    BocRefNotFound = 206,
    InvalidBocRef = 207,
}
pub struct Error;

fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as u32, message)
}

impl Error {
    pub fn missing_source_boc() -> ClientError {
        error(
            ErrorCode::MissingSourceBoc,
            "Parsed value hasn't source `boc` field".into(),
        )
    }

    pub fn invalid_boc<E: Display>(err: E) -> ClientError {
        error(ErrorCode::InvalidBoc, format!("Invalid BOC: {}", err))
    }

    pub fn serialization_error<E: Display>(err: E, name: &str) -> ClientError {
        error(
            ErrorCode::SerializationError,
            format!("Cannot serialize {}: {}", name, err),
        )
    }

    pub fn inappropriate_block<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::InappropriateBlock,
            format!("Inappropriate block: {}", err),
        )
    }

    pub fn insufficient_cache_size(max_cache_size: usize, boc_size: usize) -> ClientError {
        let mut error = error(
            ErrorCode::InsufficientCacheSize,
            "Can not insert BOC into cache: insufficient cache size".to_owned(),
        );
        error.data["max_cache_size"] = max_cache_size.into();
        error.data["boc_size"] = boc_size.into();
        error
    }

    pub fn boc_ref_not_found(boc_ref: &str) -> ClientError {
        let mut error = error(
            ErrorCode::BocRefNotFound,
            "BOC reference not found in cache".to_owned(),
        );
        error.data["boc_ref"] = boc_ref.into();
        error
    }

    pub fn invalid_boc_ref<E: Display>(err: E, boc_ref: &str) -> ClientError {
        let mut error = error(
            ErrorCode::InvalidBocRef,
            format!("Invalid BOC reference: {}", err),
        );
        error.data["boc_ref"] = boc_ref.into();
        error
    }
}
