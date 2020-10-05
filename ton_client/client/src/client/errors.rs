use crate::error::ApiError;
use std::fmt::Display;
const CLIENT: isize = ApiError::CLIENT; // 0

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
}
pub struct Error;

fn error(code: ErrorCode, message: String) -> ApiError {
    ApiError::with_code_message(code as isize, message)
}

impl Error {
    pub fn not_implemented(message: &str) -> ApiError {
        error(ErrorCode::NotImplemented, message.into())
    }

    pub fn invalid_hex<E: Display>(s: &str, err: E) -> ApiError {
        error(
            ErrorCode::InvalidHex,
            format!("Invalid hex string: {}\r\nhex: [{}]", err, s),
        )
    }

    pub fn invalid_base64<E: Display>(s: &str, err: E) -> ApiError {
        error(
            ErrorCode::InvalidBase64,
            format!("Invalid base64 string: {}\r\nbase64: [{}]", err, s),
        )
    }

    pub fn invalid_address<E: Display>(err: E, address: &str) -> ApiError {
        error(
            ErrorCode::InvalidAddress,
            format!("Invalid address [{}]: {}", err, address),
        )
    }

    pub fn callback_params_cant_be_converted_to_json<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::CallbackParamsCantBeConvertedToJson,
            format!("Callback params can't be converted to json: {}", err),
        )
    }

    pub fn websocket_connect_error<E: Display>(address: &str, err: E) -> ApiError {
        error(
            ErrorCode::WebsocketConnectError,
            format!("Can not connect to webscocket URL {}: {}", address, err),
        )
    }

    pub fn websocket_receive_error<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::WebsocketReceiveError,
            format!("Can not receive message from websocket : {}", err),
        )
    }

    pub fn websocket_send_error<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::WebsocketSendError,
            format!("Can not send message to websocket : {}", err),
        )
    }

    pub fn http_client_create_error<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::HttpClientCreateError,
            format!("Can not create http client : {}", err),
        )
    }

    pub fn http_request_create_error<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::HttpRequestCreateError,
            format!("Can not create http request : {}", err),
        )
    }

    pub fn http_request_send_error<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::HttpRequestSendError,
            format!("Can not send http request : {}", err),
        )
    }

    pub fn http_request_parse_error<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::HttpRequestParseError,
            format!("Can not parse http request : {}", err),
        )
    }

    pub fn callback_not_registered(callback_id: u32) -> ApiError {
        error(
            ErrorCode::CallbackNotRegistered,
            format!("Callback with ID {} is not registered", callback_id)
        )
    }

    pub fn net_module_not_init() -> ApiError {
        error(
            ErrorCode::NetModuleNotInit,
            "SDK is initialized without network config".to_owned(),
        )
    }

    pub fn invalid_config(message: String) -> ApiError {
        error(
            ErrorCode::InvalidConfig,
            message,
        )
    }

    pub fn cannot_create_runtime<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::CannotCreateRuntime,
            format!("Can not create runtime: {}", err)
        )
    }

    pub fn invalid_context_handle(context: u32) -> ApiError {
        error(
            ErrorCode::InvalidContextHandle,
            format!("Invalid context handle: {}", context)
        )
    }

    pub fn cannot_serialize_result() -> String {
        r#"{"source": "client", "code": 18, "message": "Can not serialize result"}"#.to_owned()
    }

    pub fn cannot_serialize_error() -> String {
        r#"{"source": "client", "code": 19, "message": "Can not serialize error"}"#.to_owned()
    }
}
