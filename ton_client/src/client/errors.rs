use crate::crypto::keys::strip_secret;
use crate::error::ClientError;
use std::fmt::{Debug, Display};

#[derive(ApiType)]
pub enum ErrorCode {
    NotImplemented = 1,
    InvalidHex = 2,
    InvalidBase64 = 3,
    InvalidAddress = 4,
    CallbackParamsCantBeConvertedToJson = 5,
    WebsocketConnectError = 6,
    WebsocketReceiveError = 7,
    WebsocketSendError = 8,
    HttpClientCreateError = 9,
    HttpRequestCreateError = 10,
    HttpRequestSendError = 11,
    HttpRequestParseError = 12,
    CallbackNotRegistered = 13,
    NetModuleNotInit = 14,
    InvalidConfig = 15,
    CannotCreateRuntime = 16,
    InvalidContextHandle = 17,
    CannotSerializeResult = 18,
    CannotSerializeError = 19,
    CannotConvertJsValueToJson = 20,
    CannotReceiveSpawnedResult = 21,
    SetTimerError = 22,
    InvalidParams = 23,
    ContractsAddressConversionFailed = 24,
    UnknownFunction = 25,
    AppRequestError = 26,
    NoSuchRequest = 27,
    CanNotSendRequestResult = 28,
    CanNotReceiveRequestResult = 29,
    CanNotParseRequestResult = 30,
    UnexpectedCallbackResponse = 31,
    CanNotParseNumber = 32,
    InternalError = 33,
    InvalidHandle = 34,
}
pub struct Error;

fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as u32, message)
}

pub const CANNOT_SERIALIZE_RESULT: &str = r#"{ "code": 18, "message": "Can not serialize result"}"#;

lazy_static! {
    static ref SECRET_REGEX: regex::Regex =
        regex::Regex::new(r#""secret"\s*:\s*"([0-9a-f]{64})""#).unwrap();
}

impl Error {
    pub fn is_network_error(error: &ClientError) -> bool {
        error.code == ErrorCode::WebsocketConnectError as u32
            || error.code == ErrorCode::WebsocketReceiveError as u32
            || error.code == ErrorCode::WebsocketSendError as u32
            || error.code == ErrorCode::HttpRequestSendError as u32
            || (error.code == crate::net::ErrorCode::GraphqlError as u32
                && error.data["server_code"].as_i64() >= Some(500)
                && error.data["server_code"].as_i64() <= Some(599)
            )
    }

    pub fn internal_error<E: Display>(message: E) -> ClientError {
        error(ErrorCode::InternalError, message.to_string())
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
        let mut params_json_stripped = params_json.to_owned();
        while let Some(captures) = SECRET_REGEX.captures(&params_json_stripped) {
            let key = captures.get(1).unwrap().as_str();
            let stripped = strip_secret(key);
            params_json_stripped = params_json_stripped.replace(key, stripped.as_str());
        }

        error(
            ErrorCode::InvalidParams,
            format!(
                "Invalid parameters: {}\nparams: {}",
                err, params_json_stripped
            ),
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
            format!(
                "Can not send request result. Probably receiver is already dropped. Request ID {}",
                id
            ),
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
            format!(
                "Unexpected callback response. Expected {}, received {:#?}",
                expected, received
            ),
        )
    }

    pub fn can_not_parse_number(string: &str) -> ClientError {
        error(
            ErrorCode::CanNotParseNumber,
            format!("Can not parse integer from string `{}`", string),
        )
    }

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

    pub fn invalid_handle(handle: u32, name: &str) -> ClientError {
        error(
            ErrorCode::InvalidHandle,
            format!("Invalid {} handle: {}", name, handle),
        )
    }
}
