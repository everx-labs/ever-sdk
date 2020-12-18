use crate::error::ClientError;
use std::fmt::{Debug, Display};
const CLIENT: isize = ClientError::CLIENT; // 0

pub enum ErrorCode {
    NotImplemented = CLIENT + 1,
    InvalidHex = CLIENT + 2,
    InvalidBase64 = CLIENT + 3,
    InvalidAddress = CLIENT + 4,
    CallbackParamsCantBeConvertedToJson = CLIENT + 5,
    WebsocketConnectError = CLIENT + 6,
    WebsocketReceiveError = CLIENT + 7,
    WebsocketSendError = CLIENT + 8,
    HttpClientCreateError = CLIENT + 9,
    HttpRequestCreateError = CLIENT + 10,
    HttpRequestSendError = CLIENT + 11,
    HttpRequestParseError = CLIENT + 12,
    CallbackNotRegistered = CLIENT + 13,
    NetModuleNotInit = CLIENT + 14,
    InvalidConfig = CLIENT + 15,
    CannotCreateRuntime = CLIENT + 16,
    InvalidContextHandle = CLIENT + 17,
    CannotSerializeResult = CLIENT + 18,
    CannotSerializeError = CLIENT + 19,
    CannotConvertJsValueToJson = CLIENT + 20,
    CannotReceiveSpawnedResult = CLIENT + 21,
    SetTimerError = CLIENT + 22,
    InvalidParams = CLIENT + 23,
    ContractsAddressConversionFailed = CLIENT + 24,
    UnknownFunction = CLIENT + 25,
    AppRequestError = CLIENT + 26,
    NoSuchRequest = CLIENT + 27,
    CanNotSendRequestResult = CLIENT + 28,
    CanNotReceiveRequestResult = CLIENT + 29,
    CanNotParseRequestResult = CLIENT + 30,
    UnexpectedCallbackResponse = CLIENT + 31,
    CanNotParseNumber = CLIENT + 32,
    InternalError = CLIENT + 33,
}
pub struct Error;

fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as u32, message)
}

pub const CANNOT_SERIALIZE_RESULT: &str = r#"{ "code": 18, "message": "Can not serialize result"}"#;

impl Error {
    pub fn is_network_error(error: &ClientError) -> bool {
        error.code == ErrorCode::WebsocketConnectError as u32
            || error.code == ErrorCode::WebsocketReceiveError as u32
            || error.code == ErrorCode::WebsocketSendError as u32
            || error.code == ErrorCode::HttpRequestSendError as u32
    }

    pub fn internal_error(message: &str) -> ClientError {
        error(ErrorCode::InternalError, message.into())
    }

    pub fn not_implemented(message: &str) -> ClientError {
        error(ErrorCode::NotImplemented, message.into())
    }

    pub fn invalid_hex<E: Display>(s: &str, err: E) -> ClientError {
        error(
            ErrorCode::InvalidHex,
            format!("Invalid hex string: {}\r\nhex: [{}]", err, s),
        )
    }

    pub fn invalid_base64<E: Display>(s: &str, err: E) -> ClientError {
        error(
            ErrorCode::InvalidBase64,
            format!("Invalid base64 string: {}\r\nbase64: [{}]", err, s),
        )
    }

    pub fn invalid_address<E: Display>(err: E, address: &str) -> ClientError {
        error(
            ErrorCode::InvalidAddress,
            format!("Invalid address [{}]: {}", err, address),
        )
    }

    pub fn callback_params_cant_be_converted_to_json<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::CallbackParamsCantBeConvertedToJson,
            format!("Callback params can't be converted to json: {}", err),
        )
    }

    pub fn websocket_connect_error<E: Display>(address: &str, err: E) -> ClientError {
        error(
            ErrorCode::WebsocketConnectError,
            format!("Can not connect to webscocket URL {}: {}", address, err),
        )
    }

    pub fn websocket_receive_error<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::WebsocketReceiveError,
            format!("Can not receive message from websocket: {}", err),
        )
    }

    pub fn websocket_send_error<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::WebsocketSendError,
            format!("Can not send message to websocket: {}", err),
        )
    }

    pub fn http_client_create_error<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::HttpClientCreateError,
            format!("Can not create http client: {}", err),
        )
    }

    pub fn http_request_create_error<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::HttpRequestCreateError,
            format!("Can not create http request: {}", err),
        )
    }

    pub fn http_request_send_error<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::HttpRequestSendError,
            format!("Can not send http request: {}", err),
        )
    }

    pub fn http_request_parse_error<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::HttpRequestParseError,
            format!("Can not parse http request: {}", err),
        )
    }

    pub fn callback_not_registered(callback_id: u32) -> ClientError {
        error(
            ErrorCode::CallbackNotRegistered,
            format!("Callback with ID {} is not registered", callback_id),
        )
    }

    pub fn net_module_not_init() -> ClientError {
        error(
            ErrorCode::NetModuleNotInit,
            "SDK is initialized without network config".to_owned(),
        )
    }

    pub fn invalid_config(message: String) -> ClientError {
        error(ErrorCode::InvalidConfig, message)
    }

    pub fn cannot_create_runtime<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::CannotCreateRuntime,
            format!("Can not create runtime: {}", err),
        )
    }

    pub fn invalid_context_handle(context: u32) -> ClientError {
        error(
            ErrorCode::InvalidContextHandle,
            format!("Invalid context handle: {}", context),
        )
    }

    pub fn cannot_serialize_result(err: impl Display) -> ClientError {
        error(
            ErrorCode::CannotSerializeResult,
            format!("Can't serialize result: {}", err),
        )
    }

    pub fn invalid_params(params_json: &str, err: impl Display) -> ClientError {
        error(
            ErrorCode::InvalidParams,
            format!("Invalid parameters: {}\nparams: {}", err, params_json),
        )
    }

    pub fn contracts_address_conversion_failed(err: impl Display) -> ClientError {
        error(
            ErrorCode::ContractsAddressConversionFailed,
            format!("Address conversion failed: {}", err),
        )
    }

    pub fn unknown_function(name: &str) -> ClientError {
        error(
            ErrorCode::UnknownFunction,
            format!("Unknown function: {}", name),
        )
    }

    pub fn app_request_error(text: &str) -> ClientError {
        error(
            ErrorCode::AppRequestError,
            format!("Application request returned error: {}", text),
        )
    }

    pub fn no_such_request(id: u32) -> ClientError {
        error(
            ErrorCode::NoSuchRequest,
            format!("No such request. ID {}", id),
        )
    }

    pub fn can_not_send_request_result(id: u32) -> ClientError {
        error(
            ErrorCode::CanNotSendRequestResult,
            format!("Can not send request result. Probably receiver is already dropped. Request ID {}", id),
        )
    }

    pub fn can_not_receive_request_result(err: impl Display) -> ClientError {
        error(
            ErrorCode::CanNotReceiveRequestResult,
            format!("Can not receive request result: {}", err),
        )
    }

    pub fn can_not_parse_request_result(err: impl Display) -> ClientError {
        error(
            ErrorCode::CanNotParseRequestResult,
            format!("Can not parse request result: {}", err),
        )
    }

    pub fn unexpected_callback_response(expected: &str, received: impl Debug) -> ClientError {
        error(
            ErrorCode::UnexpectedCallbackResponse,
            format!("Unexpected callback response. Expected {}, received {:#?}", expected, received),
        )
    }

    pub fn can_not_parse_number(string: &str) -> ClientError {
        error(
            ErrorCode::CanNotParseNumber,
            format!("Can not parse integer from string `{}`", string),
        )
    }
}

impl Error {
    pub fn cannot_convert_jsvalue_to_json(value: impl std::fmt::Debug) -> ClientError {
        error(
            ErrorCode::CannotConvertJsValueToJson,
            format!("Can not convert JS value to JSON: {:#?}", value),
        )
    }

    pub fn can_not_receive_spawned_result(err: impl Display) -> ClientError {
        error(
            ErrorCode::CannotReceiveSpawnedResult,
            format!("Can not receive result from spawned task: {}", err),
        )
    }

    pub fn set_timer_error(err: impl Display) -> ClientError {
        error(
            ErrorCode::SetTimerError,
            format!("Set timer error: {}", err),
        )
    }
}
