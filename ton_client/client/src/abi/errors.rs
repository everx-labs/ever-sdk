use crate::error::ApiError;
use std::fmt::Display;
const ABI: isize = ApiError::ABI; // 200

pub enum Code {
    RequiredAddressMissingForEncodeMessage = ABI + 0,
    RequiredCallSetMissingForEncodeMessage = ABI + 1,
    InvalidJson = ABI + 2,
    InvalidMessage = ABI + 3,
    EncodeDeployMessageFailed = ABI + 4,
    EncodeRunMessageFailed = ABI + 5,
    AttachSignatureFailed = ABI + 6,
    InvalidTvcImage = ABI + 7,
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
