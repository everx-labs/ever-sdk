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
 *
 */

use crate::error::ClientError;
use chrono::TimeZone;
use serde_json::Value;

const PROCESSING: isize = ClientError::PROCESSING; // 500

pub enum ErrorCode {
    MessageAlreadyExpired = PROCESSING + 1,
    MessageHasNotDestinationAddress = PROCESSING + 2,
    CanNotBuildMessageCell = PROCESSING + 3,
    FetchBlockFailed = PROCESSING + 4,
    SendMessageFailed = PROCESSING + 5,
    InvalidMessageBoc = PROCESSING + 6,
    MessageExpired = PROCESSING + 7,
    TransactionWaitTimeout = PROCESSING + 8,
    InvalidBlockReceived = PROCESSING + 9,
    CanNotCheckBlockShard = PROCESSING + 10,
    BlockNotFound = PROCESSING + 11,
    InvalidData = PROCESSING + 12,
    ExternalSignerMustNotBeUsed = PROCESSING + 13,
}

pub struct Error;

fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as u32, message)
}

fn error_with_data(code: ErrorCode, message: String, data: Value) -> ClientError {
    ClientError::new(code as u32, message, data)
}

fn format_time(time: u32) -> String {
    format!(
        "{} ({})",
        chrono::Local.timestamp(time as i64, 0).to_rfc2822(),
        time
    )
}

impl Error {
    pub fn external_signer_must_not_be_used() -> ClientError {
        error(
            ErrorCode::ExternalSignerMustNotBeUsed,
            "Function `process_message` must not be used with external message signing.".into(),
        )
    }

    pub fn message_already_expired() -> ClientError {
        error(
            ErrorCode::MessageAlreadyExpired,
            "Message can’t be sent because it is expired".into(),
        )
    }

    pub fn message_has_not_destination_address() -> ClientError {
        error(
            ErrorCode::MessageHasNotDestinationAddress,
            "Message can't be sent because it hasn't destination address".into(),
        )
    }

    fn processing_error(
        code: ErrorCode,
        message: String,
        message_id: &str,
        shard_block_id: Option<&String>,
    ) -> ClientError {
        let mut data = json!({
            "message_id": message_id,
        });
        if let Some(shard_block_id) = shard_block_id {
            data["shard_block_id"] = shard_block_id.clone().into();
        }
        error_with_data(code, message, data)
    }

    pub fn fetch_first_block_failed<E: std::fmt::Display>(err: E, message_id: &str) -> ClientError {
        Self::processing_error(
            ErrorCode::FetchBlockFailed,
            format!("Fetch first block failed: {}", err),
            message_id,
            None,
        )
    }

    pub fn fetch_block_failed<E: std::fmt::Display>(
        err: E,
        message_id: &str,
        shard_block_id: &String,
    ) -> ClientError {
        Self::processing_error(
            ErrorCode::FetchBlockFailed,
            format!("Fetch block failed: {}", err),
            message_id,
            Some(shard_block_id),
        )
    }

    pub fn send_message_failed<E: std::fmt::Display>(
        err: E,
        message_id: &str,
        shard_block_id: &String,
    ) -> ClientError {
        Self::processing_error(
            ErrorCode::SendMessageFailed,
            format!("Send message failed: {}", err),
            message_id,
            Some(shard_block_id),
        )
    }

    pub fn invalid_message_boc<E: std::fmt::Display>(err: E) -> ClientError {
        error(
            ErrorCode::InvalidMessageBoc,
            format!("Invalid message BOC: {}", err),
        )
    }

    pub fn can_not_build_message_cell<E: std::fmt::Display>(err: E) -> ClientError {
        error(
            ErrorCode::CanNotBuildMessageCell,
            format!("Can't build message cell: {}", err),
        )
    }

    pub fn invalid_block_received<E: std::fmt::Display>(
        err: E,
        message_id: &str,
        shard_block_id: &String,
    ) -> ClientError {
        Self::processing_error(
            ErrorCode::InvalidBlockReceived,
            format!("Invalid block received: {}", err),
            message_id,
            Some(shard_block_id),
        )
    }

    pub fn fetch_transaction_result_failed<E: std::fmt::Display>(
        err: E,
        message_id: &str,
        shard_block_id: &String,
    ) -> ClientError {
        Self::processing_error(
            ErrorCode::InvalidBlockReceived,
            err.to_string(),
            message_id,
            Some(shard_block_id),
        )
    }

    pub fn message_expired(
        message_id: &str,
        shard_block_id: &String,
        expiration_time: u32,
        block_time: u32,
    ) -> ClientError {
        let mut error = Self::processing_error(
            ErrorCode::MessageExpired,
            "Message expired".into(),
            message_id,
            Some(shard_block_id),
        );

        error.data["waiting_expiration_time"] = format_time(expiration_time).into();
        error.data["block_time"] = format_time(block_time).into();

        error
    }

    pub fn transaction_wait_timeout(
        message_id: &str,
        shard_block_id: &String,
        expiration_time: u32,
        timeout: u32,
        block_time: u32,
    ) -> ClientError {
        let mut error = Self::processing_error(
            ErrorCode::TransactionWaitTimeout,
            "Transaction wait timeout".into(),
            message_id,
            Some(shard_block_id),
        );

        error.data["waiting_expiration_time"] = format_time(expiration_time).into();
        error.data["timeout"] = timeout.into();
        error.data["block_time"] = format_time(block_time).into();

        error
    }

    pub fn can_not_check_block_shard<E: std::fmt::Display>(err: E) -> ClientError {
        error(
            ErrorCode::CanNotCheckBlockShard,
            format!("Can't check block shard: {}", err),
        )
    }

    pub fn block_not_found(message: String) -> ClientError {
        error(ErrorCode::BlockNotFound, message)
    }

    pub fn invalid_data<E: std::fmt::Display>(err: E) -> ClientError {
        error(ErrorCode::InvalidData, format!("Invalid data: {}", err))
    }
}
