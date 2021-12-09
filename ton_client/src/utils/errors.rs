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
