use crate::error::ClientError;
use std::fmt::Display;

#[derive(ApiType)]
pub enum ErrorCode {
    RequiredAddressMissingForEncodeMessage = 301,
    RequiredCallSetMissingForEncodeMessage = 302,
    InvalidJson = 303,
    InvalidMessage = 304,
    EncodeDeployMessageFailed = 305,
    EncodeRunMessageFailed = 306,
    AttachSignatureFailed = 307,
    InvalidTvcImage = 308,
    RequiredPublicKeyMissingForFunctionHeader = 309,
    InvalidSigner = 310,
    InvalidAbi = 311,
    InvalidFunctionId = 312,
}

pub struct Error;

fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as u32, message)
}

impl Error {
    pub fn invalid_abi<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::InvalidAbi,
            format!("Invalid ABI specified: {}", err),
        )
    }

    pub fn invalid_signer(message: String) -> ClientError {
        error(ErrorCode::InvalidSigner, message.into())
    }

    pub fn required_address_missing_for_encode_message() -> ClientError {
        error(
            ErrorCode::RequiredAddressMissingForEncodeMessage,
            "Address must be provided to encode run message.".into(),
        )
    }

    pub fn missing_required_call_set_for_encode_message() -> ClientError {
        error(
            ErrorCode::RequiredCallSetMissingForEncodeMessage,
            "Call parameters must be provided to encode run message.".into(),
        )
    }

    pub fn required_public_key_missing_for_function_header() -> ClientError {
        error(
            ErrorCode::RequiredPublicKeyMissingForFunctionHeader,
            "Public key must be provided to encode function header.".into(),
        )
    }

    pub fn invalid_json<E: Display>(err: E) -> ClientError {
        error(ErrorCode::InvalidJson, format!("Invalid ABI JSON: {}", err))
    }

    pub fn invalid_message_for_decode<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::InvalidMessage,
            format!("Message can't be decoded: {}", err),
        )
    }

    pub fn encode_deploy_message_failed<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::EncodeDeployMessageFailed,
            format!("Encode deploy message failed: {}", err),
        )
    }

    pub fn encode_run_message_failed<E: Display>(err: E, function: &str) -> ClientError {
        error(
            ErrorCode::EncodeRunMessageFailed,
            format!("Create run message failed: {}", err),
        )
        .add_function(Some(function))
    }

    pub fn attach_signature_failed<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::AttachSignatureFailed,
            format!("Encoding message with sign failed: {}", err),
        )
    }

    pub fn invalid_tvc_image<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::InvalidTvcImage,
            format!("Invalid TVC image: {}", err),
        )
    }

    pub fn invalid_function_id<E: Display>(func_id: &str, err: E) -> ClientError {
        error(
            ErrorCode::InvalidFunctionId,
            format!("Invalid function {}: {}", func_id, err),
        )
    }
}
