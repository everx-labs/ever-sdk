use crate::error::ApiError;
use std::fmt::Display;
const ABI: isize = ApiError::ABI; // 200

pub enum ErrorCode {
    RequiredAddressMissingForEncodeMessage = ABI + 1,
    RequiredCallSetMissingForEncodeMessage = ABI + 2,
    InvalidJson = ABI + 3,
    InvalidMessage = ABI + 4,
    EncodeDeployMessageFailed = ABI + 5,
    EncodeRunMessageFailed = ABI + 6,
    AttachSignatureFailed = ABI + 7,
    InvalidTvcImage = ABI + 8,
    RequiredPublicKeyMissingForFunctionHeader = ABI + 9,
}

pub struct Error;

fn error(code: ErrorCode, message: String) -> ApiError {
    ApiError::with_code_message(code as isize, message)
}

impl Error {
    pub fn required_address_missing_for_encode_message() -> ApiError {
        error(
            ErrorCode::RequiredAddressMissingForEncodeMessage,
            "Address must be provided to encode run message.".into(),
        )
    }

    pub fn missing_required_call_set_for_encode_message() -> ApiError {
        error(
            ErrorCode::RequiredCallSetMissingForEncodeMessage,
            "Call parameters must be provided to encode run message.".into(),
        )
    }

    pub fn required_public_key_missing_for_function_header() -> ApiError {
        error(
            ErrorCode::RequiredPublicKeyMissingForFunctionHeader,
            "Public key must be provided to encode function header.".into(),
        )
    }

    pub fn invalid_json<E: Display>(err: E) -> ApiError {
        error(ErrorCode::InvalidJson, format!("Invalid ABI JSON: {}", err))
    }

    pub fn invalid_message_for_decode<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::InvalidMessage,
            format!("Message can't be decoded: {}", err),
        )
    }

    pub fn encode_deploy_message_failed<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::EncodeDeployMessageFailed,
            format!("Encode deploy message failed: {}", err),
        )
    }

    pub fn encode_run_message_failed<E: Display>(err: E, function: &str) -> ApiError {
        error(
            ErrorCode::EncodeRunMessageFailed,
            format!("Create run message failed: {}", err),
        )
        .add_function(Some(function))
    }

    pub fn attach_signature_failed<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::AttachSignatureFailed,
            format!("Encoding message with sign failed: {}", err),
        )
    }

    pub fn invalid_tvc_image<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::InvalidTvcImage,
            format!("Invalid TVC image: {}", err),
        )
    }
}
