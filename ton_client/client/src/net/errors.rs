use crate::error::ApiError;
use std::fmt::Display;
const NET: isize = ApiError::NET; // 500

pub enum ErrorCode {
    Base = NET,
    InvalidServerResponse = NET + 50,
    ClockOutOfSync = NET + 51,
    WaitForTimeout = NET + 52,
    GraphqlError = NET + 53,
}

pub struct Error;

fn error(code: ErrorCode, message: String) -> ApiError {
    ApiError::with_code_message(code as isize, message)
}

impl Error {
    pub fn invalid_server_response<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::InvalidServerResponse,
            format!("Invalid server response : {}", err),
        )
    }

    pub fn clock_out_of_sync(delta_ms: i64, threshold: i64) -> ApiError {
        let mut error = error(
            ErrorCode::ClockOutOfSync,
            "The time on the device is out of sync with the time on the server".to_owned(),
        );

        error.data = serde_json::json!({
            "delta_ms": delta_ms,
            "threshold_ms": threshold,
            "tip": "Synchronize your device time with internet time"
        });
        error
    }

    pub fn wait_for_timeout() -> ApiError {
        error(
            ErrorCode::WaitForTimeout,
            "wait_for operation did not return anything during the specified timeout".to_owned()
        )
    }

    pub fn graphql_error<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::GraphqlError,
            format!("Graphql server returned error: {}", err)
        )
    }
}
