use crate::error::ClientError;
use std::fmt::Display;
const NET: isize = ClientError::NET; // 600

pub enum ErrorCode {
    QueryFailed = NET + 1,
    SubscribeFailed = NET + 2,
    WaitForFailed = NET + 3,
    GetSubscriptionResultFailed = NET + 4,
    InvalidServerResponse = NET + 5,
    ClockOutOfSync = NET + 6,
    WaitForTimeout = NET + 7,
    GraphqlError = NET + 8,
}
pub struct Error;

fn error(code: ErrorCode, message: String) -> ClientError {
    ClientError::with_code_message(code as u32, message)
}

impl Error {
    pub fn queries_query_failed<E: Display>(err: E) -> ClientError {
        error(ErrorCode::QueryFailed, format!("Query failed: {}", err))
    }

    pub fn queries_subscribe_failed<E: Display>(err: E) -> ClientError {
        error(ErrorCode::SubscribeFailed, format!("Subscribe failed: {}", err))
    }

    pub fn queries_wait_for_failed<E: Display>(err: E) -> ClientError {
        error(ErrorCode::WaitForFailed, format!("WaitFor failed: {}", err))
    }

    pub fn queries_get_subscription_result_failed<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::GetSubscriptionResultFailed,
            format!("Receive subscription result failed: {}", err),
        )
    }

    pub fn invalid_server_response<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::InvalidServerResponse,
            format!("Invalid server response : {}", err),
        )
    }

    pub fn clock_out_of_sync(delta_ms: i64, threshold: u32) -> ClientError {
        let mut error = error(
            ErrorCode::ClockOutOfSync,
            "The time on the device is out of sync with the time on the server. Synchronize your device time with internet time".to_owned(),
        );

        error.data = serde_json::json!({
            "delta_ms": delta_ms,
            "threshold_ms": threshold,
        });
        error
    }

    pub fn wait_for_timeout() -> ClientError {
        error(
            ErrorCode::WaitForTimeout,
            "wait_for operation did not return anything during the specified timeout".to_owned()
        )
    }

    pub fn graphql_error<E: Display>(err: E) -> ClientError {
        error(
            ErrorCode::GraphqlError,
            format!("Graphql server returned error: {}", err)
        )
    }
}
