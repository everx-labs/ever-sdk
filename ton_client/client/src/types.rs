#![allow(dead_code)]

use std::fmt::Display;
use types::ApiSdkErrorCode::*;

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
    SDK,
    TVM,
    StdLib,
    Contract,
}

impl ApiErrorSource {
    pub fn to_string(&self) -> String {
        match self {
            ApiErrorSource::SDK => "sdk".to_string(),
            ApiErrorSource::TVM => "tvm".to_string(),
            ApiErrorSource::StdLib => "stdlib".to_string(),
            ApiErrorSource::Contract => "contract".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiError {
    pub source: String,
    pub code: usize,
    pub message: String,
}

pub type ApiResult<T> = Result<T, ApiError>;

trait ApiErrorCode {
    fn as_number(&self) -> usize;
}


macro_rules! sdk_err {
    ($code:expr, $($args:tt),*) => (
        ApiError::new(ApiErrorSource::SDK, &$code, format!($($args),*))
    );
}

impl ApiError {
    fn new(source: ApiErrorSource, code: &ApiErrorCode, message: String) -> Self {
        Self {
            source: source.to_string(),
            code: code.as_number(),
            message,
        }
    }

    pub fn sdk(code: ApiSdkErrorCode, message: String) -> Self {
        Self::new(ApiErrorSource::SDK, &code, message)
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

    pub fn crypto_invalid_secret_key<E: Display>(err: E, key: &String) -> Self {
        sdk_err!(CryptoInvalidSecretKey,
            "Invalid secret key [{}]: {}", err, key)
    }

    pub fn crypto_invalid_public_key<E: Display>(err: E, key: &String) -> Self {
        sdk_err!(CryptoInvalidPublicKey,
            "Invalid secret key [{}]: {}", err, key)
    }

    pub fn crypto_invalid_address<E: Display>(err: E, address: &String) -> Self {
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

    pub fn contracts_run_contract_not_found() -> ApiError {
        Self::sdk(ContractsRunContractNotFound, "Contract not found".into())
    }

    pub fn contracts_deploy_invalid_image<E: Display>(err: E) -> Self {
        sdk_err!(ContractsDeployInvalidImage,
            "Invalid contract image: {}", err)
    }

    pub fn contracts_deploy_image_creation_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsDeployImageCreationFailed,
            "Image creation failed: {}", err)
    }

    pub fn contracts_deploy_transaction_missing() -> Self {
        ApiError::sdk(ContractsDeployTransactionMissing,
        "Deploy failed: transaction missing".into())
    }

    pub fn contracts_encode_message_with_sign_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsEncodeMessageWithSignFailed,
            "Encoding message with sign failed: {}", err)
    }

    // TVM

    pub fn tvm_execution_skipped(reason: u8) -> ApiError {
        ApiError::new(
            ApiErrorSource::TVM,
            &ApiTvmErrorCode::ExecutionSkipped,
            format!("Contract execution skipped with reason: {}", reason)
        )
    }

    pub fn tvm_execution_failed(exit_code: i32) -> ApiError {
        ApiError::new(
            ApiErrorSource::Contract,
            &ApiContractErrorCode { exit_code },
            format!("Contract execution failed with VM exit code: {}", exit_code)
        )
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

    ContractsLoadFailed = 3001,
    ContractsDeployInvalidImage = 3002,
    ContractsDeployImageCreationFailed = 3003,
    ContractsDeployTransactionMissing = 3004,
    ContractsDecodeRunOutputFailed = 3005,
    ContractsDecodeRunInputFailed = 3006,
    ContractsRunContractNotFound = 3008,
    ContractsRunTransactionMissing = 3009,
    ContractsSendMessageFailed = 3010,
    ContractsCreateDeployMessageFailed = 3011,
    ContractsCreateRunMessageFailed = 3012,
    ContractsCreateSendGramsMessageFailed = 3013,
    ContractsEncodeMessageWithSignFailed = 3014,

    Requests = 4000,

    Wallet = 5000,

}

impl ApiErrorCode for ApiSdkErrorCode {
    fn as_number(&self) -> usize {
        (self.clone() as i32) as usize
    }
}

#[derive(Clone)]
pub enum ApiTvmErrorCode {
    ExecutionSkipped = 3006,
    ExecutionFailed = 3007,
}

impl ApiErrorCode for ApiTvmErrorCode {
    fn as_number(&self) -> usize {
        (self.clone() as i32) as usize
    }
}


#[derive(Clone)]
pub enum ApiStdLibErrorCode {
    NoError = 0
}

impl ApiErrorCode for ApiStdLibErrorCode {
    fn as_number(&self) -> usize {
        (self.clone() as i32) as usize
    }
}

pub struct ApiContractErrorCode {
    exit_code: i32
}

impl ApiErrorCode for ApiContractErrorCode {
    fn as_number(&self) -> usize {
        self.exit_code as usize
    }
}

