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
    DebotStartFailed = 801,
    DebotFetchFailed = 802,
    DebotExecutionFailed = 803,
    DebotInvalidHandle = 804,
    DebotInvalidJsonParams = 805,
    DebotInvalidFunctionId = 806,
    DebotInvalidAbi = 807,
    DebotGetMethodFailed = 808,
    DebotInvalidMsg = 809,
    DebotExternalCallFailed = 810,
    DebotBrowserCallbackFailed = 811,
    DebotOperationRejected = 812,
    DebotNoCode = 813,
}
pub struct Error;

pub fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as u32, message)
}

impl Error {
    pub fn start_failed(err: impl Display) -> ClientError {
        error(
            ErrorCode::DebotStartFailed,
            format!("Debot start failed: {}", err),
        )
    }

    pub fn fetch_failed(err: impl Display) -> ClientError {
        error(
            ErrorCode::DebotFetchFailed,
            format!("Debot fetch failed: {}", err),
        )
    }

    pub fn execute_failed(err: impl Display) -> ClientError {
        error(
            ErrorCode::DebotExecutionFailed,
            format!("Debot execution failed: {}", err),
        )
    }

    pub fn invalid_handle(handle: u32) -> ClientError {
        error(
            ErrorCode::DebotInvalidHandle,
            format!("Invalid debot handle: {}", handle),
        )
    }

    pub fn invalid_json_params(err: impl Display) -> ClientError {
        error(
            ErrorCode::DebotInvalidJsonParams,
            format!("Invalid json parameters: {}", err),
        )
    }

    pub fn invalid_function_id(err: impl Display) -> ClientError {
        error(
            ErrorCode::DebotInvalidFunctionId,
            format!("Invalid function id: {}", err),
        )
    }

    pub fn invalid_debot_abi(err: impl Display) -> ClientError {
        error(
            ErrorCode::DebotInvalidAbi,
            format!("Invalid debot ABI: {}", err),
        )
    }

    pub fn get_method_failed(err: impl Display) -> ClientError {
        error(
            ErrorCode::DebotGetMethodFailed,
            format!("get-method call failed: {}", err),
        )
    }

    pub fn invalid_msg(err: impl Display) -> ClientError {
        error(ErrorCode::DebotInvalidMsg, format!("invalid msg ({})", err))
    }

    pub fn external_call_failed(err: impl Display) -> ClientError {
        error(
            ErrorCode::DebotExternalCallFailed,
            format!("external call failed: ({})", err),
        )
    }

    pub fn operation_rejected() -> ClientError {
        error(
            ErrorCode::DebotOperationRejected,
            format!("Debot operation was rejected by user"),
        )
    }

    pub fn browser_callback_failed(err: impl Display) -> ClientError {
        error(
            ErrorCode::DebotBrowserCallbackFailed,
            format!("Debot browser callback failed: {}", err),
        )
    }

    pub fn debot_has_no_code() -> ClientError {
        error(ErrorCode::DebotNoCode, format!("Debot has no code"))
    }
}
