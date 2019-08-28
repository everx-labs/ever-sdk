use serde::export::fmt::Display;
use error::ErrorCode::*;
use InteropContext;

#[derive(Deserialize, Serialize)]
pub(crate) struct ClientError {
    pub category: String,
    pub code: u32,
    pub message: String,
}

impl ClientError {
    pub fn sdk(code: ErrorCode, message: &str) -> Self {
        Self {
            category: "sdk".to_string(),
            code: code as u32,
            message: message.to_string(),
        }
    }

    pub fn interop_failed<E: Display>(err: E) -> Self {
        Self::sdk(InteropFailed, &format!("Interop failed: {}", err))
    }

    pub fn setup_failed<E: Display>(err: E) -> Self {
        Self::sdk(SetupFailed, &format!("Setup failed: {}", err))
    }

    pub fn invalid_context_handle(context: InteropContext) -> Self {
        Self::sdk(InvalidContextHandle, &format!("Setup failed: {}", context))
    }

    pub fn invalid_params<E: Display>(params_json: &str, err: E) -> Self {
        Self::sdk(InvalidParams, &format!("Invalid params: {}\nparams: [{}]", err, params_json))
    }

    pub fn unknown_method(method: &str) -> Self {
        Self::sdk(UnknownMethod, &format!("Unknown method: {}", method))
    }

}

pub(crate) enum ErrorCode {
    InteropFailed = 1,
    InvalidContextHandle = 2,
    UnknownMethod = 3,
    InvalidParams = 4,
    SetupFailed = 5,
}

pub(crate) type ClientResult<R> = Result<R, ClientError>;
