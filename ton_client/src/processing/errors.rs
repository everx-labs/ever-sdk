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
 *
 */

use crate::error::{ClientError, format_time};
use serde_json::Value;
use ton_block::MsgAddressInt;

#[derive(ApiType)]
pub enum ErrorCode {
    MessageAlreadyExpired = 501,
    MessageHasNotDestinationAddress = 502,
    CanNotBuildMessageCell = 503,
    FetchBlockFailed = 504,
    SendMessageFailed = 505,
    InvalidMessageBoc = 506,
    MessageExpired = 507,
    TransactionWaitTimeout = 508,
    InvalidBlockReceived = 509,
    CanNotCheckBlockShard = 510,
    BlockNotFound = 511,
    InvalidData = 512,
    ExternalSignerMustNotBeUsed = 513,
    MessageRejected = 514,
    InvalidRempStatus = 515,
    NextRempStatusTimeout = 516,
}

pub struct Error;

fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as u32, message)
}

fn error_with_data(code: ErrorCode, message: String, data: Value) -> ClientError {
    ClientError::new(code as u32, message, data)
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
        shard_block_id: Option<&str>,
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
        shard_block_id: &str,
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
        shard_block_id: &str,
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
        shard_block_id: &str,
        expiration_time: u32,
        block_time: u32,
        address: &MsgAddressInt,
    ) -> ClientError {
        let mut error = Self::processing_error(
            ErrorCode::MessageExpired,
            "Message expired. Contract was not executed on chain.".into(),
            message_id,
            Some(shard_block_id),
        );

        error.data["waiting_expiration_time"] = format_time(expiration_time).into();
        error.data["block_time"] = format_time(block_time).into();
        error.data["account_address"] = address.to_string().into();

        error
    }

    pub fn transaction_wait_timeout(
        message_id: &str,
        shard_block_id: &String,
        expiration_time: u32,
        timeout: u32,
        block_time: u32,
        address: &MsgAddressInt,
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
        error.data["account_address"] = address.to_string().into();

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

    pub fn message_rejected(message_id: &str, err: &str) -> ClientError {
        Self::processing_error(
            ErrorCode::MessageRejected,
            format!("message has been rejected: {}", err),
            message_id,
            None,
        )
    }

    pub fn invalid_remp_status<E: std::fmt::Display>(err: E) -> ClientError {
        error(ErrorCode::InvalidRempStatus, format!("Invalid REMP status: {}", err))
    }

    pub fn next_remp_status_timeout() -> ClientError {
        error(ErrorCode::NextRempStatusTimeout, format!("Next REMP status awaiting timeout"))
    }
}
