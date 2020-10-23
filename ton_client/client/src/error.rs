use crate::error::ClientSdkErrorCode::*;
use chrono::TimeZone;
use serde_json::Value;
use std::fmt::Display;
use ton_block::{AccStatusChange, ComputeSkipReason, MsgAddressInt};
use ton_sdk::MessageProcessingState;
use ton_types::ExceptionCode;

fn format_time(time: u32) -> String {
    format!(
        "{} ({})",
        chrono::Local.timestamp(time as i64, 0).to_rfc2822(),
        time
    )
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, ApiType)]
#[serde(default)]
pub struct ClientError {
    pub code: isize,
    pub message: String,
    pub data: serde_json::Value,
}

pub type ClientResult<T> = Result<T, ClientError>;

pub trait ClientErrorCode {
    fn as_number(&self) -> isize;
}

trait AsString {
    fn as_string(&self) -> String;
}

macro_rules! sdk_err {
    ($code:expr, $($args:tt),*) => (
        ClientError::with_code_message($code.as_number(), format!($($args),*))
    );
}

macro_rules! as_number_impl {
    ($name:ident) => {
        impl ClientErrorCode for $name {
            fn as_number(&self) -> isize {
                self.clone() as isize
            }
        }
    };
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ClientError {}

impl ClientError {
    pub const CLIENT: isize = 0;
    pub const CRYPTO: isize = 100;
    pub const BOC: isize = 200;
    pub const ABI: isize = 300;
    pub const TVM: isize = 400;
    pub const PROCESSING: isize = 500;
    pub const NET: isize = 600;
    pub const UTILS: isize = 700;

    pub fn new(code: isize, message: String, data: Value) -> Self {
        let mut data = data;
        data["core_version"] = Value::String(env!("CARGO_PKG_VERSION").to_owned());
        Self {
            code,
            message,
            data,
        }
    }

    pub fn with_code_message(code: isize, message: String) -> Self {
        Self {
            code,
            message,
            data: json!({
                "core_version": env!("CARGO_PKG_VERSION").to_owned(),
            }),
        }
    }

    pub fn sdk(code: ClientSdkErrorCode, message: String) -> Self {
        Self::with_code_message(code.as_number(), message)
    }

        pub(crate) fn add_network_url(mut self, client: &crate::net::NodeClient) -> ClientError {
        self.data["config_server"] = client.config_server().into();

        if let Some(url) = client.query_url() {
            self.data["query_url"] = url.into();
        }

        self
    }

    pub fn add_function(mut self, function: Option<&str>) -> ClientError {
        if let Some(function) = function {
            self.data["function_name"] = function.into();
        }

        self
    }

    pub fn add_address(mut self, address: &MsgAddressInt) -> ClientError {
        self.data["account_address"] = address.to_string().into();
        self
    }

    // SDK Common

    pub fn unknown_function(name: &String) -> ClientError {
        sdk_err!(UnknownFunction, "Unknown function [{}]", name)
    }

    pub fn invalid_params<E: Display>(params_json: &str, err: E) -> Self {
        sdk_err!(
            InvalidParams,
            "Invalid parameters: {}\nparams: [{}]",
            err,
            params_json
        )
    }

    pub fn invalid_context_handle(context: u32) -> Self {
        sdk_err!(InvalidContextHandle, "Invalid context handle: {}", context)
    }

    pub fn cannot_create_runtime<E: Display>(err: E) -> Self {
        sdk_err!(CannotCreateRuntime, "Can not create runtime: {}", err)
    }

    pub fn sdk_not_init() -> Self {
        ClientError::sdk(SdkNotInit, "SDK is not initialized".into())
    }

    // SDK Config

    pub fn config_init_failed<E: Display>(err: E) -> Self {
        sdk_err!(ConfigInitFailed, "Config init failed: {}", err)
    }

    pub fn wait_for_timeout() -> Self {
        sdk_err!(
            WaitForTimeout,
            "WaitFor operation did not return anything during the specified timeout"
        )
    }

    pub fn message_expired(
        msg_id: String,
        sending_time: u32,
        expire: u32,
        block_time: u32,
        block_id: String,
    ) -> Self {
        ClientError::new(
            ClientSdkErrorCode::MessageExpired.as_number(),
            "Message was not delivered within the specified timeout".to_owned(),
            serde_json::json!({
                "message_id": msg_id,
                "sending_time": format_time(sending_time),
                "expiration_time": format_time(expire),
                "block_time": format_time(block_time),
                "block_id": block_id,
            }),
        )
    }

    pub fn address_reqired_for_runget() -> Self {
        sdk_err!(AddressRequiredForRunGet,
            "Address is required for run local. You haven't specified contract code or data so address is required to load missing parts from network.")
    }

    pub fn network_silent(
        msg_id: String,
        timeout: u32,
        block_id: String,
        _state: MessageProcessingState,
    ) -> Self {
        ClientError::new(
            ClientSdkErrorCode::NetworkSilent.as_number(),
            "No blocks were produced during the specified timeout".to_owned(),
            serde_json::json!({
                "message_id": msg_id,
                "timeout": timeout,
                "last_block_id": block_id,
            }),
        )
    }

    pub fn transaction_wait_timeout(
        msg_id: String,
        sending_time: u32,
        timeout: u32,
        _state: MessageProcessingState,
    ) -> Self {
        ClientError::new(
            ClientSdkErrorCode::TransactionWaitTimeout.as_number(),
            "Transaction was not produced during the specified timeout".to_owned(),
            serde_json::json!({
                "message_id": msg_id,
                "sending_time": format_time(sending_time),
                "timeout": timeout,
            }),
        )
    }

    pub fn account_code_missing(address: &MsgAddressInt) -> Self {
        ClientError::new(
            ClientSdkErrorCode::AccountCodeMissing.as_number(),
            "Contract is not deployed".to_owned(),
            serde_json::json!({
                "tip": "Contract code should be deployed before calling contract functions",
                "account_address": address.to_string(),
            }),
        )
    }

    pub fn low_balance(address: &MsgAddressInt, balance: Option<u64>) -> Self {
        let mut data = serde_json::json!({
            "account_address": address.to_string(),
            "tip": "Send some value to account balance",
        });
        if let Some(balance) = balance {
            data["account_balance"] = balance.into();
        }
        ClientError::new(
            ClientSdkErrorCode::LowBalance.as_number(),
            "Account has insufficient balance for the requested operation".to_owned(),
            data,
        )
    }

    pub fn account_frozen_or_deleted(address: &MsgAddressInt) -> Self {
        ClientError::new(
            ClientSdkErrorCode::AccountFrozenOrDeleted.as_number(),
            "Account is in a bad state. It is frozen or deleted".to_owned(),
            serde_json::json!({
                "account_address": address.to_string(),
            }),
        )
    }

    pub fn account_missing(address: &MsgAddressInt) -> Self {
        ClientError::new(
            ClientSdkErrorCode::AccountMissing.as_number(),
            "Account does not exist".to_owned(),
            serde_json::json!({
                "account_address": address.to_string(),
                "tip": "You need to transfer funds to this account first to have a positive balance and then deploy its code."
            }),
        )
    }

    pub fn clock_out_of_sync(delta_ms: i64, threshold: i64) -> Self {
        ClientError::new(
            ClientSdkErrorCode::ClockOutOfSync.as_number(),
            "The time on the device is out of sync with the time on the server".to_owned(),
            serde_json::json!({
                "delta_ms": delta_ms,
                "threshold_ms": threshold,
                "tip": "Synchronize your device time with internet time"
            }),
        )
    }

    pub fn callback_not_registered(callback_id: u32) -> Self {
        sdk_err!(
            CallbackNotRegistered,
            "Callback with ID {} is not registered",
            callback_id
        )
    }

    // SDK Cell

    pub fn cell_invalid_query<E: Display>(s: E) -> Self {
        sdk_err!(CellInvalidQuery, "Invalid cell query: {}", s)
    }

    // SDK Contracts

    pub fn contracts_load_failed<E: Display>(err: E, address: &String) -> Self {
        sdk_err!(
            ContractsLoadFailed,
            "Load contract [{}] failed: {}",
            address,
            err
        )
    }

    pub fn contracts_send_message_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsSendMessageFailed, "Send message failed: {}", err)
    }

    pub fn contracts_create_deploy_message_failed<E: Display>(err: E) -> Self {
        sdk_err!(
            ContractsCreateDeployMessageFailed,
            "Create deploy message failed: {}",
            err
        )
    }

    pub fn contracts_create_run_message_failed<E: Display>(err: E, function: &str) -> Self {
        sdk_err!(
            ContractsCreateRunMessageFailed,
            "Create run message failed: {}",
            err
        )
        .add_function(Some(function))
    }

    pub fn contracts_create_send_grams_message_failed<E: Display>(err: E) -> Self {
        sdk_err!(
            ContractsCreateSendGramsMessageFailed,
            "Create send grams message failed: {}",
            err
        )
    }

    pub fn contracts_decode_run_output_failed<E: Display>(err: E, function: Option<&str>) -> Self {
        sdk_err!(
            ContractsDecodeRunOutputFailed,
            "Decode run output failed: {}",
            err
        )
        .add_function(function)
    }

    pub fn contracts_decode_run_input_failed<E: Display>(err: E, function: Option<&str>) -> Self {
        sdk_err!(
            ContractsDecodeRunInputFailed,
            "Decode run input failed: {}",
            err
        )
        .add_function(function)
    }

    pub fn contracts_run_failed<E: Display>(err: E) -> ClientError {
        sdk_err!(ContractsRunFailed, "Contract run failed: {}", err)
    }

    pub fn contracts_run_contract_load_failed<E: Display>(err: E) -> ClientError {
        sdk_err!(
            ContractsRunContractLoadFailed,
            "Contract load failed: {}",
            err
        )
    }

    pub fn contracts_invalid_image<E: Display>(err: E) -> Self {
        sdk_err!(ContractsInvalidImage, "Invalid contract image: {}", err)
    }

    pub fn contracts_image_creation_failed<E: Display>(err: E) -> Self {
        sdk_err!(
            ContractsImageCreationFailed,
            "Image creation failed: {}",
            err
        )
    }

    pub fn contracts_deploy_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsDeployFailed, "Deploy failed: {}", err)
    }

    pub fn contracts_deploy_transaction_aborted() -> Self {
        ClientError::sdk(
            ContractsDeployTransactionAborted,
            "Deploy failed: transaction aborted".into(),
        )
    }

    pub fn contracts_run_body_creation_failed<E: Display>(err: E) -> Self {
        sdk_err!(
            ContractsRunBodyCreationFailed,
            "Run body creation failed: {}",
            err
        )
    }

    pub fn contracts_encode_message_with_sign_failed<E: Display>(err: E) -> Self {
        sdk_err!(
            ContractsEncodeMessageWithSignFailed,
            "Encoding message with sign failed: {}",
            err
        )
    }

    pub fn contracts_get_function_id_failed<E: Display>(err: E, function: &str) -> Self {
        sdk_err!(
            ContractsGetFunctionIdFailed,
            "Get function ID failed: {}",
            err
        )
        .add_function(Some(function))
    }

    pub fn contracts_local_run_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsLocalRunFailed, "Local run failed: {}", err)
    }

    pub fn contracts_address_conversion_failed<E: Display>(err: E) -> Self {
        sdk_err!(
            ContractsAddressConversionFailed,
            "Address conversion failed: {}",
            err
        )
    }

    pub fn contracts_invalid_boc<E: Display>(err: E) -> Self {
        sdk_err!(ContractsInvalidBoc, "Invalid Bag of Cells: {}", err)
    }

    pub fn contracts_load_messages_failed<E: Display>(err: E) -> Self {
        sdk_err!(ContractsLoadMessagesFailed, "Load messages failed: {}", err)
    }

    pub fn contracts_cannot_serialize_message<E: Display>(err: E) -> Self {
        sdk_err!(
            ContractsCannotSerializeMessage,
            "Can not serialize message: {}",
            err
        )
    }

    pub fn contracts_process_message_failed<E: Display>(err: E) -> Self {
        sdk_err!(
            ContractsProcessMessageFailed,
            "Process message failed: {}",
            err
        )
    }

    pub fn contracts_find_shard_failed<E: Display>(err: E) -> Self {
        sdk_err!(
            ContractsFindShardFailed,
            "Account shard search failed: {}",
            err
        )
    }

    // SDK queries

    pub fn queries_query_failed<E: Display>(err: E) -> Self {
        sdk_err!(QueriesQueryFailed, "Query failed: {}", err)
    }

    pub fn queries_subscribe_failed<E: Display>(err: E) -> Self {
        sdk_err!(QueriesSubscribeFailed, "Subscribe failed: {}", err)
    }

    pub fn queries_wait_for_failed<E: Display>(err: E) -> Self {
        sdk_err!(QueriesWaitForFailed, "WaitFor failed: {}", err)
    }

    pub fn queries_get_next_failed<E: Display>(err: E) -> Self {
        sdk_err!(
            QueriesGetSubscriptionResultFailed,
            "Receive subscription result failed: {}",
            err
        )
    }

    // Failed transaction phases

    pub fn transaction_aborted(tr_id: Option<String>) -> ClientError {
        ClientError::new(
            -1,
            "Transaction was aborted".to_string(),
            serde_json::json!({
                "transaction_id": tr_id,
                "phase": "unknown",
            }),
        )
    }

    pub fn tvm_execution_skipped(
        tr_id: Option<String>,
        reason: &ComputeSkipReason,
        address: &MsgAddressInt,
        balance: Option<u64>,
    ) -> ClientError {
        let mut error = match reason {
            ComputeSkipReason::NoState => Self::account_code_missing(address),
            ComputeSkipReason::BadState => Self::account_frozen_or_deleted(address),
            ComputeSkipReason::NoGas => Self::low_balance(address, balance),
        };

        error.data["transaction_id"] = tr_id.map(|s| s.into()).unwrap_or(serde_json::Value::Null);
        error.data["phase"] = "computeSkipped".into();

        error
    }

    pub fn tvm_execution_failed(
        tr_id: Option<String>,
        exit_code: i32,
        address: &MsgAddressInt,
    ) -> ClientError {
        let mut data = serde_json::json!({
            "transaction_id": tr_id,
            "phase": "computeVm",
            "exit_code": exit_code,
            "account_address": address.to_string()
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
        ClientError::new(
            ContractsTvmError.as_number(),
            format!("Contract execution was terminated with error"),
            data,
        )
    }

    pub fn storage_phase_failed(
        tr_id: Option<String>,
        reason: &AccStatusChange,
        address: &MsgAddressInt,
        balance: Option<u64>,
    ) -> ClientError {
        let mut error = Self::low_balance(address, balance);
        error.data["transaction_id"] = tr_id.map(|s| s.into()).unwrap_or(serde_json::Value::Null);
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
        tr_id: Option<String>,
        result_code: i32,
        valid: bool,
        no_funds: bool,
        address: &MsgAddressInt,
        balance: Option<u64>,
    ) -> ClientError {
        let mut error = if no_funds {
            let mut error = Self::low_balance(address, balance);
            error.data["description"] =
                "Contract tried to send value exceeding account balance".into();
            error
        } else {
            let mut error = ClientError::new(
                ActionPhaseFailed.as_number(),
                "Transaction failed at action phase".to_owned(),
                json!({}),
            );
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
pub enum ClientSdkErrorCode {
    UnknownFunction = 1,
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
    CallbackNotRegistered = 1020,

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
    QueriesGetSubscriptionResultFailed = 4004,

    CellInvalidQuery = 5001,
}

impl ClientErrorCode for ClientSdkErrorCode {
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
        }
        .to_string()
    }
}

as_number_impl!(AccStatusChange);

impl AsString for AccStatusChange {
    fn as_string(&self) -> String {
        match self {
            AccStatusChange::Unchanged => "Account was unchanged",
            AccStatusChange::Frozen => "Account was frozen due storage phase",
            AccStatusChange::Deleted => "Account was deleted due storage phase",
        }
        .to_string()
    }
}

pub struct ClientContractErrorCode {
    exit_code: i32,
}

impl ClientErrorCode for ClientContractErrorCode {
    fn as_number(&self) -> isize {
        self.exit_code as isize
    }
}

impl ClientErrorCode for i32 {
    fn as_number(&self) -> isize {
        self.clone() as isize
    }
}

#[cfg(feature = "node_interaction")]
pub(crate) fn _clienterror_from_sdkerror<F>(
    err: &failure::Error,
    default_err: F,
    client: Option<&crate::net::NodeClient>,
) -> ClientError
where
    F: Fn(String) -> ClientError,
{
    let err = match err.downcast_ref::<ton_sdk::SdkError>() {
        Some(ton_sdk::SdkError::WaitForTimeout) => ClientError::wait_for_timeout(),
        Some(ton_sdk::SdkError::MessageExpired {
            msg_id,
            expire,
            sending_time,
            block_time,
            block_id,
        }) => ClientError::message_expired(
            msg_id.to_string(),
            *sending_time,
            *expire,
            *block_time,
            block_id.to_string(),
        ),
        Some(ton_sdk::SdkError::NetworkSilent {
            msg_id,
            timeout,
            block_id,
            state,
        }) => ClientError::network_silent(
            msg_id.to_string(),
            *timeout,
            block_id.to_string(),
            state.clone(),
        ),
        Some(ton_sdk::SdkError::TransactionWaitTimeout {
            msg_id,
            sending_time,
            timeout,
            state,
        }) => ClientError::transaction_wait_timeout(
            msg_id.to_string(),
            *sending_time,
            *timeout,
            state.clone(),
        ),
        Some(ton_sdk::SdkError::ClockOutOfSync {
            delta_ms,
            threshold_ms,
        }) => ClientError::clock_out_of_sync(*delta_ms, *threshold_ms),
        Some(ton_sdk::SdkError::ResumableNetworkError { error, .. }) => {
            _clienterror_from_sdkerror(error, default_err, client)
        }
        _ => default_err(err.to_string()),
    };

    if let Some(client) = client {
        err.add_network_url(client)
    } else {
        err
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
