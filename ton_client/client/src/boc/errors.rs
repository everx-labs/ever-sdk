/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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
const BOC: isize = ClientError::BOC; // 200

pub enum ErrorCode {
    InvalidBoc = BOC + 1,
    SerializationError = BOC + 2,
    InappropriateBlock = BOC + 3,
    MissingSourceBoc = BOC + 4,
}
pub struct Error;

fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as isize, message)
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
}
