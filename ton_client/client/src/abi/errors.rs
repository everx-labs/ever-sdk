use crate::error::ApiError;
use std::fmt::Display;

pub enum Code {
    RequiredAddressMissingForEncodeMessage = 300,
    RequiredCallSetMissingForEncodeMessage = 301,
    InvalidJson = 302,
    InvalidMessage = 303,
    EncodeDeployMessageFailed = 304,
    EncodeRunMessageFailed = 305,
    AttachSignatureFailed = 306,
    InvalidTvcImage = 307,
}

pub struct Error;

fn error(code: Code, message: String) -> ApiError {
    ApiError::with_code_message(code as isize, message)
}

impl Error {
    pub fn required_address_missing_for_encode_message() -> ApiError {
        error(
            Code::RequiredAddressMissingForEncodeMessage,
            "Address must be provided to encode run message.".into(),
        )
    }

    pub fn missing_required_call_set_for_encode_message() -> ApiError {
        error(
            Code::RequiredCallSetMissingForEncodeMessage,
            "Call parameters must be provided to encode run message.".into(),
        )
    }

    pub fn invalid_json<E: Display>(err: E) -> ApiError {
        error(Code::InvalidJson, format!("Invalid ABI JSON: {}", err))
    }

    pub fn invalid_message_for_decode<E: Display>(err: E) -> ApiError {
        error(
            Code::InvalidMessage,
            format!("Message can't be decoded: {}", err),
        )
    }

    pub fn encode_deploy_message_failed<E: Display>(err: E) -> ApiError {
        error(
            Code::EncodeDeployMessageFailed,
            format!("Encode deploy message failed: {}", err),
        )
    }

    pub fn encode_run_message_failed<E: Display>(err: E, function: &str) -> ApiError {
        error(
            Code::EncodeRunMessageFailed,
            format!("Create run message failed: {}", err),
        )
        .add_function(Some(function))
    }

    pub fn attach_signature_failed<E: Display>(err: E) -> ApiError {
        error(
            Code::AttachSignatureFailed,
            format!("Encoding message with sign failed: {}", err),
        )
    }

    pub fn invalid_tvc_image<E: Display>(err: E) -> ApiError {
        error(Code::InvalidTvcImage, format!("Invalid TVC image: {}", err))
    }
}
