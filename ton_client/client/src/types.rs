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

#![allow(dead_code)]

use std::fmt::Display;
use ApiSdkErrorCode::*;
use ton_block::{AccStatusChange, ComputeSkipReason, MsgAddressInt};
use ton_sdk::{SdkError, MessageProcessingState};
use ton_types::ExceptionCode;
use chrono::TimeZone;

pub fn hex_decode(hex: &String) -> ApiResult<Vec<u8>> {
    if hex.starts_with("x") || hex.starts_with("X") {
        hex_decode(&hex.chars().skip(1).collect())
    } else if hex.starts_with("0x") || hex.starts_with("0X") {
        hex_decode(&hex.chars().skip(2).collect())
    } else {
        hex::decode(hex).map_err(|err| ApiError::crypto_invalid_hex(&hex, err))
    }
}

pub fn base64_decode(base64: &String) -> ApiResult<Vec<u8>> {
    base64::decode(base64).map_err(|err| ApiError::crypto_invalid_base64(&base64, err))
}

pub fn long_num_to_json_string(num: u64) -> String {
    format!("0x{:x}", num)
}

fn format_time(time: u32) -> String {
    format!("{} ({})", chrono::Local.timestamp(time as i64 , 0).to_rfc2822(), time)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ApiErrorSource {
    Client,
    Node
}

impl ApiErrorSource {
    pub fn to_string(&self) -> String {
        match self {
            ApiErrorSource::Client => "client".to_string(),
            ApiErrorSource::Node => "node".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct ApiError {
    pub core_version: String,
    pub source: String,
    pub code: isize,
    pub message: String,
    pub message_processing_state: Option<MessageProcessingState>,
    pub data: serde_json::Value
}

pub type ApiResult<T> = Result<T, ApiError>;

pub trait ApiErrorCode {
    fn as_number(&self) -> isize;
}

trait AsString {
    fn as_string(&self) -> String;
}

macro_rules! sdk_err {
    ($code:expr, $($args:tt),*) => (
        ApiError::new(ApiErrorSource::Client, &$code, format!($($args),*))
    );
}

macro_rules! as_number_impl {
    ($name:ident) => (
        impl ApiErrorCode for $name {
            fn as_number(&self) -> isize {
                self.clone() as isize
            }
        }
    );
}

impl ApiError {
    fn new(source: ApiErrorSource, code: &dyn ApiErrorCode, message: String) -> Self {
        Self {
            core_version: env!("CARGO_PKG_VERSION").to_owned(),
            source: source.to_string(),
            code: code.as_number(),
            message,
            message_processing_state: None,
            data: serde_json::Value::Null,
        }
    }

    pub fn sdk(code: ApiSdkErrorCode, message: String) -> Self {
        Self::new(ApiErrorSource::Client, &code, message)
    }

    // SDK Common

    pub fn unknown_method(method: &String) -> ApiError {
        sdk_err!(UnknownMethod,
            "Unknown method [{}]", method)
    }

    pub fn invalid_params<E: Display>(params_json: &str, err: E) -> Self {
        sdk_err!(InvalidParams,
            "Invalid parameters: {}\nparams: [{}]", err, params_json)
    }

    pub fn invalid_context_handle(context: u32) -> Self {
        sdk_err!(InvalidContextHandle,
            "Invalid context handle: {}", context)
    }

    pub fn cannot_create_runtime<E: Display>(err: E) -> Self {
        sdk_err!(CannotCreateRuntime,
            "Can not create runtime: {}", err)
    }

    pub fn sdk_not_init() -> Self {
        ApiError::sdk(SdkNotInit,
            "SDK is not initialized".into())
    }


    // SDK Config

    pub fn config_init_failed<E: Display>(err: E) -> Self {
        sdk_err!(ConfigInitFailed,
            "Config init failed: {}", err)
    }

    pub fn wait_for_timeout() -> Self {
        sdk_err!(WaitForTimeout,
            "WaitFor operation did not return anything during the specified timeout")
    }

    pub fn message_expired(msg_id: String, sending_time: u32, expire: u32, block_time: u32, block_id: String) -> Self {
        let mut error = ApiError::new(
            ApiErrorSource::Node,
            &ApiSdkErrorCode::MessageExpired,
            "Message was not delivered within the specified timeout".to_owned(),
        );

        error.data = serde_json::json!({
            "message_id": msg_id,
            "sending_time": format_time(sending_time),
            "expiration_time": format_time(expire),
            "block_time": format_time(block_time),
            "block_id": block_id,
        });
        error
    }

    pub fn address_reqired_for_runget() -> Self {
        sdk_err!(AddressRequiredForRunGet,
            "Address is required for run local. You haven't specified contract code or data so address is required to load missing parts from network.")
    }

    pub fn network_silent(msg_id: String, timeout: u32, block_id: String, state: MessageProcessingState) -> Self {
        let mut error = ApiError::new(
            ApiErrorSource::Node,
            &ApiSdkErrorCode::NetworkSilent,
            "No blocks were produced during the specified timeout".to_owned(),
        );
        error.message_processing_state = Some(state);

        error.data = serde_json::json!({
            "message_id": msg_id,
            "timeout": timeout,
            "last_block_id": block_id,
        });
        error
    }

    pub fn transaction_wait_timeout(msg_id: String, sending_time: u32, timeout: u32, state: MessageProcessingState) -> Self {
        let mut error = ApiError::new(
            ApiErrorSource::Node,
            &ApiSdkErrorCode::TransactionWaitTimeout,
            "Transaction was not produced during the specified timeout".to_owned(),
        );
        error.message_processing_state = Some(state);

        error.data = serde_json::json!({
            "message_id": msg_id,
            "sending_time": format_time(sending_time),
            "timeout": timeout,
        });
        error
    }

    pub fn account_code_missing(address: &MsgAddressInt) -> Self {
        let mut error = ApiError::new(
            ApiErrorSource::Node,
            &ApiSdkErrorCode::AccountCodeMissing,
            "Contract is not deployed".to_owned(),
        );

        error.data = serde_json::json!({
            "tip": "Contract code should be deployed before calling contract functions",
            "address": address.to_string(),
        });
        error
    }

    pub fn low_balance(address: &MsgAddressInt) -> Self {
        let mut error = ApiError::new(
            ApiErrorSource::Node,
            &ApiSdkErrorCode::LowBalance,
            "Account has insufficient balance for the requested operation".to_owned(),
        );

        error.data = serde_json::json!({
            "address": address.to_string(),
            "tip": "Send some value to account balance",
        });
        error
    }

    pub fn account_frozen_or_deleted(address: &MsgAddressInt) -> Self {
        let mut error = ApiError::new(
            ApiErrorSource::Node,
            &ApiSdkErrorCode::AccountFrozenOrDeleted,
            "Account is in a bad state. It is frozen or deleted".to_owned(),
        );

        error.data = serde_json::json!({
            "address": address.to_string(),
        });
        error
    }

    pub fn account_missing(address: &MsgAddressInt) -> Self {
        let mut error = ApiError::new(
            ApiErrorSource::Node,
            &ApiSdkErrorCode::AccountMissing,
            "Account does not exist".to_owned(),
        );

        error.data = serde_json::json!({
            "address": address.to_string(),
            "tip": "You need to transfer funds to this account first to have a positive balance and then deploy its code."
        });
        error
    }

    pub fn clock_out_of_sync(delta_ms: i64, threshold: i64, expiration_timout: u32) -> Self {
        let mut error = ApiError::new(
            ApiErrorSource::Node,
            &ApiSdkErrorCode::ClockOutOfSync,
            "The time on the device is out of sync with the time on the server".to_owned(),
        );

        error.data = serde_json::json!({
            "delta_ms": delta_ms,
            "threshold_ms": threshold,
            "expiration_timout_ms": expiration_timout,
            "tip": "Synchronize your device time with internet time"
        });
        error
    }

    // SDK Cell

    pub fn cell_invalid_query<E: Display>(s: E) -> Self {
        sdk_err!(CellInvalidQuery,
            "Invalid cell query: {}", s)
    }

    // SDK Crypto

    pub fn crypto_invalid_hex<E: Display>(s: &String, err: E) -> Self {
        sdk_err!(CryptoInvalidHex,
            "Invalid hex string: {}\r\nhex: [{}]", err, s)
    }

    pub fn crypto_invalid_base64<E: Display>(s: &String, err: E) -> Self {
        sdk_err!(CryptoInvalidHex,
            "Invalid base64 string: {}\r\nbase64: [{}]", err, s)
    }

    pub fn crypto_invalid_factorize_challenge<E: Display>(hex: &String, err: E) -> Self {
        sdk_err!(CryptoInvalidFactorizeChallenge,
            "Invalid factorize challenge: {}\r\nchallenge: [{}]", err, hex)
    }

    pub fn crypto_invalid_big_int(hex: &String) -> Self {
        sdk_err!(CryptoInvalidBigInt,
            "Invalid big int [{}]", hex)
    }

    pub fn crypto_convert_input_data_missing() -> Self {
        Self::sdk(CryptoConvertInputDataMissing,
            r#"Input data for conversion function is missing. Expected one of { text: "..." }, { hex: "..." } or { base64: "..." }"#.to_string())
    }
    pub fn crypto_convert_output_can_not_be_encoded_to_utf8<E: Display>(err: E) -> Self {
        sdk_err!(CryptoConvertOutputCanNotBeEncodedToUtf8,
            r#"Output data for conversion function can not be encoded to utf8: {}"#,
            err)
    }

    pub fn crypto_scrypt_failed<E: Display>(err: E) -> Self {
        sdk_err!(CryptoScryptFailed,
            r#"Scrypt failed: {}"#, err)
    }

    pub fn crypto_invalid_key_size(actual: usize, expected: usize) -> Self {
        sdk_err!(CryptoInvalidKeySize,
            "Invalid key size {}. Expected {}.", actual, expected)
    }

    pub fn crypto_nacl_secret_box_failed<E: Display>(err: E) -> Self {
        sdk_err!(CryptoNaclSecretBoxFailed,
            "Secretbox failed: {}", err)
    }

    pub fn crypto_nacl_box_failed<E: Display>(err: E) -> Self {
        sdk_err!(CryptoNaclBoxFailed,
            "Box failed: {}", err)
    }

    pub fn crypto_nacl_sign_failed<E: Display>(err: E) -> Self {
        sdk_err!(CryptoNaclSignFailed,
            "Sign failed: {}", err)
    }

    pub fn crypto_bip39_invalid_entropy<E: Display>(err: E) -> Self {
        sdk_err!(CryptoBip39InvalidEntropy,
            "Invalid bip39 entropy: {}", err)
    }

    pub fn crypto_bip39_invalid_phrase<E: Display>(err: E) -> Self {
        sdk_err!(CryptoBip39InvalidPhrase,
            "Invalid bip39 phrase: {}", err)
    }

    pub fn crypto_bip32_invalid_key<E: Display>(key: E) -> Self {
        sdk_err!(CryptoBip32InvalidKey,
            "Invalid bip32 key: {}", key)
    }

    pub fn crypto_bip32_invalid_derive_path<E: Display>(path: E) -> Self {
        sdk_err!(CryptoBip32InvalidDerivePath,
            "Invalid bip32 derive path: {}", path)
    }

    pub fn crypto_bip39_invalid_dictionary(dictionary: u8) -> Self {
        sdk_err!(CryptoBip39InvalidDictionary,
            "Invalid mnemonic dictionary: {}", dictionary)
    }

    pub fn crypto_bip39_invalid_word_count(word_count: u8) -> Self {
        sdk_err!(CryptoBip39InvalidWordCount,
            "Invalid mnemonic word count: {}", word_count)
    }

    pub fn crypto_invalid_secret_key<E: Display>(err: E, key: &String) -> Self {
        sdk_err!(CryptoInvalidSecretKey,
            "Invalid secret key [{}]: {}", err, key)
    }

    pub fn crypto_invalid_public_key<E: Display>(err: E, key: &String) -> Self {
        sdk_err!(CryptoInvalidPublicKey,
            "Invalid public key [{}]: {}", err, key)
    }

    pub fn crypto_invalid_address<E: Display>(err: E, address: &str) -> Self {
        sdk_err!(CryptoInvalidAddress,
            "Invalid address [{}]: {}", err, address)
    }

    pub fn crypto_invalid_key<E: Display>(err: E, key: &String) -> Self {
        sdk_err!(CryptoInvalidKey,
            "Invalid key [{}]: {}", err, key)
    }

    pub fn crypto_invalid_keystore_handle() -> Self {
        ApiError::sdk(CryptoInvalidKeystoreHandle,
            "Keystore Handle is invalid or has removed".into())
    }

    pub fn crypto_missing_key_source() -> Self {
        ApiError::sdk(CryptoMissingKeySource,
            "Either Key or Keystore Handle must be specified".into())
    }

    pub fn crypto_mnemonic_generation_failed() -> Self {
        ApiError::sdk(CryptoMnemonicGenerationFailed,
            "Mnemonic generation failed".into())
    }

    pub fn crypto_mnemonic_from_entropy_failed(reason: &str) -> Self {
        ApiError::sdk(CryptoMnemonicFromEntropyFailed,
            format!("Mnemonic from entropy failed: {}", reason))
    }

// SDK Contracts

    pub fn contracts_load_failed<E: Display>(err: E, address: &String) -> Self {
        sdk_err!(ContractsLoadFailed,
            "Load contract [{}] failed: {}", address, err)
    }

    pub fn contracts_send_message_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsSendMessageFailed,
            "Send message failed: {}", err)
    }

    pub fn contracts_create_deploy_message_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsCreateDeployMessageFailed,
            "Create deploy message failed: {}", err)
    }

    pub fn contracts_create_run_message_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsCreateRunMessageFailed,
            "Create run message failed: {}", err)
    }

    pub fn contracts_create_send_grams_message_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsCreateSendGramsMessageFailed,
            "Create send grams message failed: {}", err)
    }

    pub fn contracts_decode_run_output_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsDecodeRunOutputFailed,
            "Decode run output failed: {}", err)
    }

    pub fn contracts_decode_run_input_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsDecodeRunInputFailed,
            "Decode run input failed: {}", err)
    }

    pub fn contracts_run_failed<E: Display>(err: E) -> ApiError {
        sdk_err!(ContractsRunFailed, "Contract run failed: {}", err)
    }

    pub fn contracts_run_contract_load_failed<E: Display>(err: E) -> ApiError {
        sdk_err!(ContractsRunContractLoadFailed,
            "Contract load failed: {}", err)
    }

    pub fn contracts_invalid_image<E: Display>(err: E) -> Self {
        sdk_err!(ContractsInvalidImage,
            "Invalid contract image: {}", err)
    }

    pub fn contracts_image_creation_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsImageCreationFailed,
            "Image creation failed: {}", err)
    }

    pub fn contracts_deploy_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsDeployFailed,
            "Deploy failed: {}", err)
    }

    pub fn contracts_deploy_transaction_aborted() -> Self {
        ApiError::sdk(ContractsDeployTransactionAborted,
        "Deploy failed: transaction aborted".into())
    }

    pub fn contracts_run_body_creation_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsRunBodyCreationFailed,
        "Run body creation failed: {}", err)
    }

    pub fn contracts_encode_message_with_sign_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsEncodeMessageWithSignFailed,
            "Encoding message with sign failed: {}", err)
    }

    pub fn contracts_get_function_id_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsGetFunctionIdFailed,
            "Get function ID failed: {}", err)
    }

    pub fn contracts_local_run_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsLocalRunFailed,
            "Local run failed: {}", err)
    }

    pub fn contracts_address_conversion_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsAddressConversionFailed,
            "Address conversion failed: {}", err)
    }

    pub fn contracts_invalid_boc<E: Display>(err: E) -> Self {
        sdk_err!(ContractsInvalidBoc,
            "Invalid Bag of Cells: {}", err)
    }

    pub fn contracts_load_messages_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsLoadMessagesFailed,
            "Load messages failed: {}", err)
    }

    pub fn contracts_cannot_serialize_message<E: Display>(err: E) -> Self {
        sdk_err!(ContractsCannotSerializeMessage,
            "Can not serialize message: {}", err)
    }

    pub fn contracts_process_message_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsProcessMessageFailed,
            "Process message failed: {}", err)
    }

    pub fn contracts_find_shard_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsFindShardFailed,
            "Account shard search failed: {}", err)
    }

    // SDK queries

    pub fn queries_query_failed<E: Display>(err: E) -> Self {
        sdk_err!(QueriesQueryFailed,
            "Query failed: {}", err)
    }

    pub fn queries_subscribe_failed<E: Display>(err: E) -> Self {
        sdk_err!(QueriesSubscribeFailed,
            "Subscribe failed: {}", err)
    }

    pub fn queries_wait_for_failed<E: Display>(err: E) -> Self {
        sdk_err!(QueriesWaitForFailed,
            "WaitFor failed: {}", err)
    }

    pub fn queries_get_next_failed<E: Display>(err: E) -> Self {
        sdk_err!(QueriesGetNextFailed,
            "Get next failed: {}", err)
    }

    // Failed transaction phases

    pub fn transaction_aborted(tr_id: Option<String>) -> ApiError {
        let mut error = ApiError::new(
            ApiErrorSource::Node,
            &(-1i32),
            "Transaction was aborted".to_string()
        );
         error.data = serde_json::json!({
            "transaction_id": tr_id,
            "phase": "unknown",
        });
        error
    }

    pub fn tvm_execution_skipped(
        tr_id: Option<String>,
        reason: &ComputeSkipReason,
        address: &MsgAddressInt,
    ) -> ApiError {
        let mut error = match reason {
            ComputeSkipReason::NoState => Self::account_code_missing(address),
            ComputeSkipReason::BadState => Self::account_frozen_or_deleted(address),
            ComputeSkipReason::NoGas => Self::low_balance(address)
        };

        error.data["transaction_id"] = tr_id.map(|s| s.into()).unwrap_or(serde_json::Value::Null);
        error.data["phase"] = "computeSkipped".into();

        error
    }

    pub fn tvm_execution_failed(tr_id: Option<String>, exit_code: i32, address: &MsgAddressInt) -> ApiError {
        let mut error = ApiError::new(
            ApiErrorSource::Node,
            &ContractsTvmError,
            format!("Contract execution was terminated with error"),
        );

        let mut data = serde_json::json!({
            "transaction_id": tr_id,
            "phase": "computeVm",
            "exit_code": exit_code,
            "address": address.to_string()
        });

        if let Some(error_code) = ExceptionCode::from_usize(exit_code as usize) {
            if error_code == ExceptionCode::OutOfGas {
                data["tip"] = "Check account balance".into();
            }
            data["description"] = error_code.to_string().into();
        } else if let Some(code) = StdContractError::from_usize(exit_code as usize) {
            if let Some(tip) = code.tip() {
                data["tip"] = tip.into();
            }
            data["description"] = code.to_string().into();
        }

        error.data = data;
        error
    }

    pub fn storage_phase_failed(
        tr_id: Option<String>,
        reason: &AccStatusChange,
        address: &MsgAddressInt,
    ) -> ApiError {
        let mut error = Self::low_balance(address);
        error.data["transaction_id"] = tr_id.map(|s| s.into()).unwrap_or(serde_json::Value::Null);
        error.data["phase"] = "storage".into();
        error.data["reason"] = match reason {
                AccStatusChange::Frozen => "Account is frozen",
                AccStatusChange::Deleted => "Account is deleted",
                _ => "null"
            }.into();
        error
    }

    pub fn action_phase_failed(
        tr_id: Option<String>,
        result_code: i32,
        valid: bool,
        no_funds: bool,
        address: &MsgAddressInt,
    ) -> ApiError {
        let mut error = if no_funds {
            let mut error = Self::low_balance(address);
            error.data["description"] = "Contract tried to send value exceeding account balance".into();
            error
        } else {
            let mut error = ApiError::new(
                ApiErrorSource::Node,
                &ActionPhaseFailed,
                "Action phase failed".to_owned());
            if !valid {
                error.data["description"] = "Contract tried to send invalid oubound message".into();
            }
            error
        };
        error.data["transaction_id"] = tr_id.map(|s| s.into()).unwrap_or(serde_json::Value::Null);
        error.data["phase"] = "action".into();
        error.data["result_code"] = result_code.into();
        error
    }
}

#[derive(Clone)]
pub enum ApiSdkErrorCode {
    UnknownMethod = 1,
    InvalidParams = 2,
    InvalidContextHandle = 3,
    CannotCreateRuntime = 4,
    SdkNotInit = 5,
    WasmUnreachableCode = 6,

    ConfigInitFailed = 1001,
    WaitForTimeout = 1003,
    MessageExpired = 1006,
    AddressRequiredForRunGet = 1009,
    NetworkSilent = 1010,
    TransactionWaitTimeout = 1012,
    ClockOutOfSync = 1013,
    AccountMissing = 1014,
    AccountCodeMissing = 1015,
    LowBalance = 1016,
    AccountFrozenOrDeleted = 1017,
    ActionPhaseFailed = 1018,
    ErrorNotResolved = 1019,

    CryptoInvalidPublicKey = 2001,
    CryptoInvalidSecretKey = 2002,
    CryptoInvalidKey = 2003,
    CryptoInvalidAddress = 2004,
    CryptoInvalidHex = 2005,
    CryptoInvalidBase64 = 2006,
    CryptoInvalidFactorizeChallenge = 2007,
    CryptoInvalidBigInt = 2008,
    CryptoConvertInputDataMissing = 2009,
    CryptoConvertOutputCanNotBeEncodedToUtf8 = 2010,
    CryptoScryptFailed = 2011,
    CryptoInvalidKeySize = 2012,
    CryptoNaclSecretBoxFailed = 2013,
    CryptoNaclBoxFailed = 2014,
    CryptoNaclSignFailed = 2015,
    CryptoBip39InvalidEntropy = 2016,
    CryptoBip39InvalidPhrase = 2017,
    CryptoBip32InvalidKey = 2018,
    CryptoBip32InvalidDerivePath = 2019,
    CryptoInvalidKeystoreHandle = 2020,
    CryptoMissingKeySource = 2021,
    CryptoBip39InvalidDictionary = 2022,
    CryptoBip39InvalidWordCount = 2023,
    CryptoMnemonicGenerationFailed = 2024,
    CryptoMnemonicFromEntropyFailed = 2025,

    ContractsLoadFailed = 3001,
    ContractsInvalidImage = 3002,
    ContractsImageCreationFailed = 3003,
    ContractsDeployFailed = 3004,
    ContractsDecodeRunOutputFailed = 3005,
    ContractsDecodeRunInputFailed = 3006,
    ContractsRunContractLoadFailed = 3008,
    ContractsRunFailed = 3009,
    ContractsSendMessageFailed = 3010,
    ContractsCreateDeployMessageFailed = 3011,
    ContractsCreateRunMessageFailed = 3012,
    ContractsCreateSendGramsMessageFailed = 3013,
    ContractsEncodeMessageWithSignFailed = 3014,
    ContractsDeployTransactionAborted = 3015,
    ContractsRunBodyCreationFailed = 3016,
    ContractsGetFunctionIdFailed = 3017,
    ContractsLocalRunFailed = 3018,
    ContractsAddressConversionFailed = 3019,
    ContractsInvalidBoc = 3020,
    ContractsLoadMessagesFailed = 3021,
    ContractsCannotSerializeMessage = 3022,
    ContractsProcessMessageFailed = 3023,
    ContractsTvmError = 3025,
    ContractsFindShardFailed = 3026,

    QueriesQueryFailed = 4001,
    QueriesSubscribeFailed = 4002,
    QueriesWaitForFailed = 4003,
    QueriesGetNextFailed = 4004,

    CellInvalidQuery = 5001,
}

impl ApiErrorCode for ApiSdkErrorCode {
    fn as_number(&self) -> isize {
        (self.clone() as i32) as isize
    }
}

as_number_impl!(ComputeSkipReason);

impl AsString for ComputeSkipReason {
    fn as_string(&self) -> String {
        match self {
            ComputeSkipReason::NoState => "Account has no code and data",
            ComputeSkipReason::BadState => "Account is in a bad state: frozen or deleted",
            ComputeSkipReason::NoGas => "No gas to execute VM",
        }.to_string()
    }
}

as_number_impl!(AccStatusChange);

impl AsString for AccStatusChange {
    fn as_string(&self) -> String {
        match self {
            AccStatusChange::Unchanged => "Account was unchanged",
            AccStatusChange::Frozen => "Account was frozen due storage phase",
            AccStatusChange::Deleted => "Account was deleted due storage phase",
        }.to_string()
    }
}


pub struct ApiContractErrorCode {
    exit_code: i32
}

impl ApiErrorCode for ApiContractErrorCode {
    fn as_number(&self) -> isize {
        self.exit_code as isize
    }
}

pub struct ApiActionCode {
    pub result_code: i32,
    pub valid: bool,
    pub no_funds: bool,
}

impl ApiErrorCode for ApiActionCode {
    fn as_number(&self) -> isize {
        self.result_code as isize
    }
}

impl ApiActionCode{
    pub fn new(result_code: i32, valid: bool, no_funds: bool) -> Self {
        Self {
            result_code,
            valid,
            no_funds,
        }
    }
    pub fn as_string(&self) -> String {
        if self.no_funds {
            "Too low balance to send an outbound message"
        } else if !self.valid {
            "Outbound message is invalid"
        } else {
            "Action phase failed"
        }.to_string()
    }
}

impl ApiErrorCode for i32 {
    fn as_number(&self) -> isize {
        self.clone() as isize
    }
}

pub fn apierror_from_sdkerror<F>(err: &failure::Error, default_err: F) -> ApiError
where
    F: Fn(String) -> ApiError,
{
    match err.downcast_ref::<SdkError>() {
        Some(SdkError::WaitForTimeout) => ApiError::wait_for_timeout(),
        Some(SdkError::MessageExpired{msg_id, expire, sending_time, block_time, block_id}) =>
            ApiError::message_expired(msg_id.to_string(), *sending_time, *expire, *block_time, block_id.to_string()),
        Some(SdkError::NetworkSilent{msg_id, timeout, block_id, state}) =>
            ApiError::network_silent(msg_id.to_string(), *timeout, block_id.to_string(), state.clone()),
        Some(SdkError::TransactionWaitTimeout{msg_id, sending_time, timeout, state}) =>
            ApiError::transaction_wait_timeout(msg_id.to_string(), *sending_time, *timeout, state.clone()),
        Some(SdkError::ClockOutOfSync{delta_ms, threshold_ms, expiration_timeout}) =>
            ApiError::clock_out_of_sync(*delta_ms, *threshold_ms, *expiration_timeout),
        Some(SdkError::ResumableNetworkError{state, error}) => {
            let mut api_error = apierror_from_sdkerror(error, default_err);
            api_error.message_processing_state = Some(state.clone());
            api_error
        }
        _ => default_err(err.to_string())
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
            StdContractError::MethodNotFound => "Check contract ABI. It may be invalid or from an old contract version",
            StdContractError::UnsupportedAbiVersion => "Check contract ABI. It may be invalid or from old contract version",
            StdContractError::PubKeyNotFound => "Contract is probably deployed incorrectly",
            StdContractError::SignNotFount => "Check call parameters. Sign keys should be passed to sign message",
            StdContractError::InvalidMsg => "Check call parameters",
            StdContractError::IndexOutOfRange => "Check call parameters. Probably contract doesn't have needed data",
            StdContractError::ConstructorAlreadyCalled => "Contract cannot be redeployed",
            StdContractError::ReplayProtection => "Try again",
            StdContractError::AddressUnpackError => "Check call parameters. Probably some address parameter is invalid (e.g. empty)",
            StdContractError::PopEmptyArray => "Check call parameters. Probably contract doesn't have needed data",
            StdContractError::ExtMessageExpired => "Try again",
            StdContractError::MsgHasNoSignButHasKey => "Check call parameters. Sign keys should be passed to sign message",
            StdContractError::NoKeyInData =>"Contract is probably deployed incorrectly",
            _ => ""
        };
        if tip.len() > 0 {
            Some(tip)
        } else {
            None
        }
    }
}
