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

use crate::boc::internal::deserialize_cell_from_base64;
use crate::error::ClientError;
use serde_json::Value;
use std::fmt::Display;
use ton_block::{AccStatusChange, ComputeSkipReason, MsgAddressInt};
use ton_types::{ExceptionCode, Cell};

#[derive(ApiType)]
pub enum ErrorCode {
    CanNotReadTransaction = 401,
    CanNotReadBlockchainConfig = 402,
    TransactionAborted = 403,
    InternalError = 404,
    ActionPhaseFailed = 405,
    AccountCodeMissing = 406,
    LowBalance = 407,
    AccountFrozenOrDeleted = 408,
    AccountMissing = 409,
    UnknownExecutionError = 410,
    InvalidInputStack = 411,
    InvalidAccountBoc = 412,
    InvalidMessageType = 413,
    ContractExecutionError = 414,
}
pub struct Error;

fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as u32, message)
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

    pub fn tvm_execution_failed<E: Display>(
        err: E,
        exit_code: i32,
        exit_arg: Option<Value>,
        address: &MsgAddressInt,
        gas_used: Option<u64>,
        show_tips: bool,
    ) -> ClientError {
        let mut err = err.to_string();
        if err.starts_with("code ") {
            err = "Unknown error".to_string();
        }
        let mut error = error(
            ErrorCode::ContractExecutionError,
            if show_tips {
                format!("Contract execution was terminated with error: {}", err)
            } else {
                err.to_string()
            },
        );

        if show_tips && !error.message.to_lowercase().contains("exit code") {
            error.message.push_str(&format!(", exit code: {}", exit_code));

            let tip = match exit_code {
                0 => Some(
                    "You either forgot to add tvm.accept() into the contract's method, or try to \
                    run a get method on-chain (and it fails because it does not have tvm.accept())."
                ),

                40 => Some(
                    "Check that:\n\
                    1. your private key suits your public key;\n\
                    2. you specified a key, but the contract doesn't expect it;\n\
                    3. you specified the correct ABI."
                ),

                52 => Some(
                    "If this error occurs in 100% cases then you specified the wrong ABI. \
                    If it appears occasionally then the contract supports timestamp-based replay \
                    protection and does not allow to call it so often (call it with 5 seconds \
                    timeout)."
                ),

                _ => None,
            };

            if let Some(tip) = tip {
                error.message.push_str(&format!("\nTip: {}", tip));
            }
        }

        error.data["phase"] = "computeVm".into();
        error.data["exit_code"] = exit_code.into();
        error.data["exit_arg"] = serde_json::json!(exit_arg);
        error.data["account_address"] = address.to_string().into();
        if let Some(gas_used) = gas_used {
            error.data["gas_used"] = gas_used.into();
        }

        if let Some(error_code) = ExceptionCode::from_usize(exit_code as usize)
            .or(ExceptionCode::from_usize(!exit_code as usize))
        {
            error.message.push_str(&format!(" ({})", error_code));
            error.data["description"] = error_code.to_string().into();
            if error_code == ExceptionCode::OutOfGas {
                error.message.push_str(". Check account balance");
                if gas_used.is_none() && exit_arg.is_some() {
                    error.data["gas_used"] = exit_arg.unwrap();
                }
            }
        } else if let Some(code) = StdContractError::from_usize(exit_code as usize) {
            error.message.push_str(&format!(" ({})", code));
            error.data["description"] = code.to_string().into();
            if let Some(tip) = code.tip() {
                error.message.push_str(". ");
                error.message.push_str(tip);
            }
        } else if let Some(ref exit_arg) = exit_arg {
            if let Some(error_message) = Self::read_error_message(exit_arg) {
                error.message.push_str(&format!(", contract error: \"{}\"", error_message));
                error.data["contract_error"] = error_message.into();
            }
        }

        if show_tips {
            error.message = error.message.trim_end_matches('.').to_string();
            error.message.push_str(
                ".\nTip: For more information about exit code check the contract source code \
                or ask the contract developer");
        }

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
                error.data["description"] = "Contract tried to send invalid outbound message".into();
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
            "Contract is not deployed. Contract should be in `Active` state to call its functions"
                .to_owned(),
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

        error.data["account_address"] = address.to_string().into();
        error.data["account_balance"] = balance.into();

        error
    }

    pub fn account_frozen_or_deleted(address: &MsgAddressInt) -> ClientError {
        let mut error = error(
            ErrorCode::AccountFrozenOrDeleted,
            "Account is in a bad state. It is frozen or deleted".to_owned(),
        );

        error.data["account_address"] = address.to_string().into();

        error
    }

    pub fn account_missing(address: &MsgAddressInt) -> ClientError {
        let mut error = error(
            ErrorCode::AccountMissing,
            "Account does not exist. You need to transfer funds to this account first to have a positive balance and then deploy its code".to_owned(),
        );

        error.data["account_address"] = address.to_string().into();

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
            "Invalid message type: external outbound messages can not be processed by the contract"
                .to_owned(),
        )
    }

    pub fn internal_error<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::InternalError,
            format!("TVM internal error: {}", err),
        )
    }

    fn read_error_message(exit_arg: &Value) -> Option<String> {
        let cell = match Self::extract_cell(exit_arg) {
            Some(cell) => cell,
            None => return None,
        };

        String::from_utf8(Self::load_boc_data(&cell)).ok()
    }

    fn extract_cell(exit_arg: &Value) -> Option<Cell> {
        let map = match exit_arg {
            Value::Object(map) => map,
            _ => return None,
        };

        if let Some(arg_type) = map.get("type") {
            match arg_type {
                Value::String(arg_type) if arg_type == "Cell" => {},
                _ => return None,
            }
        }

        let base64_value = match map.get("value") {
            Some(value) => {
                match value {
                    Value::String(base64_value) => base64_value,
                    _ => return None,
                }
            },
            None => return None,
        };

        deserialize_cell_from_base64(&base64_value, "contract_error")
            .map(|(_bytes, cell)| cell)
            .ok()
    }

    fn load_boc_data(cell: &Cell) -> Vec<u8> {
        let mut result = cell.data().to_vec()[..(cell.bit_length() >> 3)].to_vec();
        for i in 0..cell.references_count() {
            if let Ok(cell) = cell.reference(i) {
                result.append(&mut Self::load_boc_data(&cell));
            }
        }
        result
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
