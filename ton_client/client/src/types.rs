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

    pub fn contracts_deploy_transaction_aborted() -> Self {
        ApiError::sdk(ContractsDeployTransactionAborted,
        "Deploy failed: transaction aborted".into())
    }

    pub fn contracts_encode_message_with_sign_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsEncodeMessageWithSignFailed,
            "Encoding message with sign failed: {}", err)
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

    pub fn tvm_execution_skipped(tr_id: String, reason: &str) -> ApiError {
        let code = ApiComputeSkippedCode::from_reason(reason);
        let mut error = ApiError::new(ApiErrorSource::Node, &code, code.as_string());
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

    pub fn storage_phase_failed(tr_id: String, reason: &str) -> ApiError {
        let code = ApiStorageCode::from_reason(reason);
        let mut error = ApiError::new(ApiErrorSource::Node, &code, code.as_string());
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
    ContractsDeployTransactionAborted = 3015,

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

#[derive(Clone)]
pub enum ApiComputeSkippedCode {
    Unknown = 0,
    NoState = 1,
    BadState = 2,
    NoGas = 3,
}
as_number_impl!(ApiComputeSkippedCode);

impl ApiComputeSkippedCode {
    pub fn from_reason(reason: &str) -> Self {
        match reason {
            "NoState" => ApiComputeSkippedCode::NoState,
            "BadState" => ApiComputeSkippedCode::BadState,
            "NoGas" => ApiComputeSkippedCode::NoGas,
            _ => ApiComputeSkippedCode::Unknown,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            ApiComputeSkippedCode::NoState => "Account has no code and data",
            ApiComputeSkippedCode::BadState => "Account has bad state: frozen or deleted",
            ApiComputeSkippedCode::NoGas => "No gas to execute VM",
            ApiComputeSkippedCode::Unknown => "Phase skipped by unknown reason",
        }.to_string()
    }
}

#[derive(Clone)]
pub enum ApiStorageCode {
    Unknown = 0,
    Unchanged = 1,
    Frozen = 2,
    Deleted = 3,
}
as_number_impl!(ApiStorageCode);

impl ApiStorageCode {
    pub fn from_reason(reason: &str) -> Self {
        match reason {
            "Unchanged" => ApiStorageCode::Unchanged,
            "Frozen" => ApiStorageCode::Frozen,
            "Deleted" => ApiStorageCode::Deleted,
            _ => ApiStorageCode::Unknown,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            ApiStorageCode::Unchanged => "Account unchanged",
            ApiStorageCode::Frozen => "Account was frozen due storage phase",
            ApiStorageCode::Deleted => "Account was deleted due storage phase",
            ApiStorageCode::Unknown => "Storage phase failed",
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
