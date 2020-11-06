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
use ton_block::{AccStatusChange, ComputeSkipReason, MsgAddressInt};
use ton_types::ExceptionCode;
use serde_json::Value;

const TVM: isize = ClientError::TVM; // 400

pub enum ErrorCode {
    CanNotReadTransaction = TVM + 1,
    CanNotReadBlockchainConfig = TVM + 2,
    TransactionAborted = TVM + 3,
    InternalError = TVM + 4,
    ActionPhaseFailed = TVM + 5,
    AccountCodeMissing = TVM + 6,
    LowBalance = TVM + 7,
    AccountFrozenOrDeleted = TVM + 8,
    AccountMissing = TVM + 9,
    UnknownExecutionError = TVM + 10,
    InvalidInputStack = TVM + 11,
    InvalidAccountBoc = TVM + 12,
    InvalidMessageType = TVM + 13,
    ContractExecutionError = TVM + 14,
}
pub struct Error;

fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as isize, message)
}

impl Error {
    pub fn invalid_input_stack<E: Display>(err: E, stack: &Value) -> ClientError {
        error(
            ErrorCode::InvalidInputStack,
            format!("Invalid JSON value for stack item ({}): {}", stack, err),
        )
    }
    pub fn invalid_account_boc<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::InvalidAccountBoc,
            format!("Invalid account BOC: {}", err),
        )
    }
    pub fn can_not_read_transaction<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::CanNotReadTransaction,
            format!("Can not read transaction: {}", err),
        )
    }

    pub fn can_not_read_blockchain_config<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::CanNotReadBlockchainConfig,
            format!("Can not read blockchain config: {}", err),
        )
    }

    pub fn transaction_aborted() -> ClientError {
        let error = error(
            ErrorCode::TransactionAborted,
            "Transaction was aborted by unknown reason".to_string(),
        );
        error
    }

    pub fn tvm_execution_skipped(
        reason: &ComputeSkipReason,
        address: &MsgAddressInt,
        balance: u64,
    ) -> ClientError {
        let mut error = match reason {
            ComputeSkipReason::NoState => Self::account_code_missing(address),
            ComputeSkipReason::BadState => Self::account_frozen_or_deleted(address),
            ComputeSkipReason::NoGas => Self::low_balance(address, balance),
        };

        error.data["phase"] = "computeSkipped".into();

        error
    }

    pub fn tvm_execution_failed<E: Display>(err: E, exit_code: i32, exit_arg: Option<Value>, address: &MsgAddressInt) -> ClientError {
        let mut error = error(
            ErrorCode::ContractExecutionError,
            format!("Contract execution was terminated with error: {}", err),
        );

        let mut data = serde_json::json!({
            "phase": "computeVm",
            "exit_code": exit_code,
            "exit_arg": exit_arg,
            "account_address": address.to_string()
        });

        if let Some(error_code) = ExceptionCode::from_usize(exit_code as usize)
            .or(ExceptionCode::from_usize(!exit_code as usize))
        {
            if error_code == ExceptionCode::OutOfGas {
                error.message.push_str(". Check account balance");
            }
            data["description"] = error_code.to_string().into();
        } else if let Some(code) = StdContractError::from_usize(exit_code as usize) {
            if let Some(tip) = code.tip() {
                error.message.push_str(". ");
                error.message.push_str(tip);
            }
            data["description"] = code.to_string().into();
        }

        error.data = data;
        error
    }

    pub fn storage_phase_failed(
        reason: &AccStatusChange,
        address: &MsgAddressInt,
        balance: u64,
    ) -> ClientError {
        let mut error = Self::low_balance(address, balance);
        error.data["phase"] = "storage".into();
        error.data["reason"] = match reason {
            AccStatusChange::Frozen => "Account is frozen",
            AccStatusChange::Deleted => "Account is deleted",
            _ => "null",
        }
        .into();
        error
    }

    pub fn action_phase_failed(
        result_code: i32,
        valid: bool,
        no_funds: bool,
        address: &MsgAddressInt,
        balance: u64,
    ) -> ClientError {
        let mut error = if no_funds {
            let mut error = Self::low_balance(address, balance);
            error.data["description"] =
                "Contract tried to send value exceeding account balance".into();
            error
        } else {
            let mut error = error(
                ErrorCode::ActionPhaseFailed,
                "Transaction failed at action phase".to_owned(),
            );
            if !valid {
                error.data["description"] = "Contract tried to send invalid oubound message".into();
            }
            error
        };
        error.data["phase"] = "action".into();
        error.data["result_code"] = result_code.into();
        error
    }

    pub fn account_code_missing(address: &MsgAddressInt) -> ClientError {
        let mut error = error(
            ErrorCode::AccountCodeMissing,
            "Contract is not deployed. Contract should be in `Active` state to call its functions".to_owned(),
        );

        error.data = serde_json::json!({
            "account_address": address.to_string(),
        });
        error
    }

    pub fn low_balance(address: &MsgAddressInt, balance: u64) -> ClientError {
        let mut error = error(
            ErrorCode::LowBalance,
            "Account has insufficient balance for the requested operation. Send some value to account balance".to_owned(),
        );

        error.data = serde_json::json!({
            "account_address": address.to_string(),
            "account_balance": balance
        });
        error
    }

    pub fn account_frozen_or_deleted(address: &MsgAddressInt) -> ClientError {
        let mut error = error(
            ErrorCode::AccountFrozenOrDeleted,
            "Account is in a bad state. It is frozen or deleted".to_owned(),
        );

        error.data = serde_json::json!({
            "account_address": address.to_string(),
        });
        error
    }

    pub fn account_missing(address: &MsgAddressInt) -> ClientError {
        let mut error = error(
            ErrorCode::AccountMissing,
            "Account does not exist. You need to transfer funds to this account first to have a positive balance and then deploy its code".to_owned(),
        );

        error.data = serde_json::json!({
            "account_address": address.to_string(),
        });
        error
    }

    pub fn unknown_execution_error<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::UnknownExecutionError,
            format!("Transaction execution failed with unknown error: {}", err),
        )
    }

    pub fn invalid_message_type() -> ClientError {
        error(
            ErrorCode::InvalidMessageType,
            "Invalid message type: external outbound messages can not be processed by the contract".to_owned(),
        )
    }

    pub fn internal_error<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::InternalError,
            format!("TVM internal error: {}", err),
        )
    }
}

#[derive(Clone, Copy, Debug, num_derive::FromPrimitive, PartialEq, failure::Fail)]
pub enum StdContractError {
    #[fail(display = "Invalid signature")]
    InvalidSignature = 40,
    #[fail(display = "Requested method was not found in the contract")]
    MethodNotFound = 41,
    #[fail(display = "Dictionary of methods was not found")]
    MethodsDictNotFound = 42,
    #[fail(display = "Unsupported ABI version")]
    UnsupportedAbiVersion = 43,
    #[fail(display = "Public key was not found in persistent data")]
    PubKeyNotFound = 44,
    #[fail(display = "Signature was not found in the message")]
    SignNotFount = 45,
    #[fail(display = "Global data dictionary is invalid")]
    DataDictInvalid = 46,
    #[fail(display = "Smart contract info was not found")]
    ScInfoNotFound = 47,
    #[fail(display = "Invalid inbound message")]
    InvalidMsg = 48,
    #[fail(display = "Invalid state of persistent data")]
    InvalidDataState = 49,
    #[fail(display = "Array index is out of range")]
    IndexOutOfRange = 50,
    #[fail(display = "Constructor was already called")]
    ConstructorAlreadyCalled = 51,
    #[fail(display = "Replay protection exception")]
    ReplayProtection = 52,
    #[fail(display = "Address unpack error")]
    AddressUnpackError = 53,
    #[fail(display = "Pop from empty array")]
    PopEmptyArray = 54,
    #[fail(display = "Bad StateInit cell for tvm_insert_pubkey. Data was not found.")]
    DataNotFound = 55,
    #[fail(display = "map.pollFisrt() for empty map")]
    PollEmptyMap = 56,
    #[fail(display = "External inbound message is expired")]
    ExtMessageExpired = 57,
    #[fail(display = "External inbound message has no signature but has public key")]
    MsgHasNoSignButHasKey = 58,
    #[fail(display = "Contract has no receive or no fallback functions")]
    NoFallback = 59,
    #[fail(display = "Contract has no fallback function but function ID is wrong")]
    NoFallbackIdWrong = 60,
    #[fail(display = "No public key in persistent data")]
    NoKeyInData = 61,
}

impl StdContractError {
    pub fn from_usize(number: usize) -> Option<StdContractError> {
        num_traits::FromPrimitive::from_usize(number)
    }

    pub fn tip(&self) -> Option<&str> {
        let tip = match self {
            StdContractError::InvalidSignature => "Check sign keys",
            StdContractError::MethodNotFound => {
                "Check contract ABI. It may be invalid or from an old contract version"
            }
            StdContractError::UnsupportedAbiVersion => {
                "Check contract ABI. It may be invalid or from old contract version"
            }
            StdContractError::PubKeyNotFound => "Contract is probably deployed incorrectly",
            StdContractError::SignNotFount => {
                "Check call parameters. Sign keys should be passed to sign message"
            }
            StdContractError::InvalidMsg => "Check call parameters",
            StdContractError::IndexOutOfRange => {
                "Check call parameters. Probably contract doesn't have needed data"
            }
            StdContractError::ConstructorAlreadyCalled => "Contract cannot be redeployed",
            StdContractError::ReplayProtection => "Try again",
            StdContractError::AddressUnpackError => {
                "Check call parameters. Probably some address parameter is invalid (e.g. empty)"
            }
            StdContractError::PopEmptyArray => {
                "Check call parameters. Probably contract doesn't have needed data"
            }
            StdContractError::ExtMessageExpired => "Try again",
            StdContractError::MsgHasNoSignButHasKey => {
                "Check call parameters. Sign keys should be passed to sign message"
            }
            StdContractError::NoKeyInData => "Contract is probably deployed incorrectly",
            _ => "",
        };
        if tip.len() > 0 {
            Some(tip)
        } else {
            None
        }
    }
}
