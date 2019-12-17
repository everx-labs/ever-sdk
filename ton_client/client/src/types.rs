/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

#![allow(dead_code)]

use std::fmt::Display;
use types::ApiSdkErrorCode::*;
use ton_block::{AccStatusChange, ComputeSkipReason};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiErrorData {
    pub transaction_id: String,
    pub phase: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiError {
    pub source: String,
    pub code: isize,
    pub message: String,
    pub data: Option<ApiErrorData>
}

pub type ApiResult<T> = Result<T, ApiError>;

trait ApiErrorCode {
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
            source: source.to_string(),
            code: code.as_number(),
            message,
            data: None,
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
            "Invalid params: {}\nparams: [{}]", err, params_json)
    }

    pub fn invalid_context_handle(context: u32) -> Self {
        sdk_err!(InvalidContextHandle,
            "Invalid context handle: {}", context)
    }



    // SDK Config

    pub fn config_init_failed<E: Display>(err: E) -> Self {
        sdk_err!(ConfigInitFailed,
            "Config init failed: {}", err)
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
            "Invalid secret key [{}]: {}", err, key)
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
            "Mnemonic generation failed (this must never be)".into())
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
            "Decode run intput failed: {}", err)
    }

    pub fn contracts_run_transaction_missing() -> ApiError {
        Self::sdk(ContractsRunTransactionMissing, "Transaction missing".into())
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

    pub fn contracts_deploy_transaction_missing() -> Self {
        ApiError::sdk(ContractsDeployTransactionMissing,
        "Deploy failed: transaction missing".into())
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

    pub fn transaction_parse_failed() -> ApiError {
        ApiError::new(
            ApiErrorSource::Node,
            &(0i32),
            "Failed to analyze transaction".to_string()
        )
    }

    pub fn transaction_aborted(tr_id: String) -> ApiError {
        let mut error = ApiError::new(
            ApiErrorSource::Node,
            &(-1i32),
            "Transaction aborted".to_string()
        );
         error.data = Some(ApiErrorData{
            transaction_id: tr_id,
            phase: "unknown".to_string(),
        });
        error
    }

    pub fn tvm_execution_skipped(tr_id: String, reason: &ComputeSkipReason) -> ApiError {
        let mut error = ApiError::new(ApiErrorSource::Node, reason, reason.as_string());
        error.data = Some(ApiErrorData{
            transaction_id: tr_id,
            phase: "computeSkipped".to_string(),
        });
        error
    }

    pub fn tvm_execution_failed(tr_id: String, exit_code: i32) -> ApiError {
        let mut error = ApiError::new(
            ApiErrorSource::Node,
            &ApiContractErrorCode { exit_code },
            format!("VM terminated with exit code: {}", exit_code),
        );

        error.data = Some(ApiErrorData{
            transaction_id: tr_id,
            phase: "computeVm".to_string(),
        });
        error
    }

    pub fn storage_phase_failed(tr_id: String, reason: &AccStatusChange) -> ApiError {
        let mut error = ApiError::new(ApiErrorSource::Node, reason, reason.as_string());
        error.data = Some(ApiErrorData{
            transaction_id: tr_id,
            phase: "storage".to_string(),
        });
        error
    }

    pub fn action_phase_failed(
        tr_id: String,
        result_code: i32,
        valid: bool,
        no_funds: bool,
    ) -> ApiError {
        let code = ApiActionCode::new(result_code, valid, no_funds);
        let mut error = ApiError::new(ApiErrorSource::Node, &code, code.as_string());
        error.data = Some(ApiErrorData{
            transaction_id: tr_id,
            phase: "action".to_string(),
        });
        error
    }
}

#[derive(Clone)]
pub enum ApiSdkErrorCode {
    UnknownMethod = 1,
    InvalidParams = 2,
    InvalidContextHandle = 3,

    ConfigInitFailed = 1001,

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
    ContractsDeployTransactionMissing = 3004,
    ContractsDecodeRunOutputFailed = 3005,
    ContractsDecodeRunInputFailed = 3006,
    ContractsRunContractLoadFailed = 3008,
    ContractsRunTransactionMissing = 3009,
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

    QueriesQueryFailed = 4001,
    QueriesSubscribeFailed = 4002,
    QueriesWaitForFailed = 4003,
    QueriesGetNextFailed = 4004,

    Wallet = 5000,

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
            ComputeSkipReason::BadState => "Account has bad state: frozen or deleted",
            ComputeSkipReason::NoGas => "No gas to execute VM",
        }.to_string()
    }
}

as_number_impl!(AccStatusChange);

impl AsString for AccStatusChange {
    fn as_string(&self) -> String {
        match self {
            AccStatusChange::Unchanged => "Account unchanged",
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
            "Too low balance to send outbound message"
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
