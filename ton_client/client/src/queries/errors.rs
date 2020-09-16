use crate::error::ApiError;
use std::fmt::Display;
const QUERIES: isize = ApiError::QUERIES; // 600

pub enum ErrorCode {
    QueryFailed = QUERIES + 1,
    SubscribeFailed = QUERIES + 2,
    WaitForFailed = QUERIES + 3,
    GetSubscriptionResultFailed = QUERIES + 4,
}
pub struct Error;

fn error(code: ErrorCode, message: String) -> ApiError {
    ApiError::with_code_message(code as isize, message)
}

impl Error {
    pub fn queries_query_failed<E: Display>(err: E) -> ApiError {
        error(ErrorCode::QueryFailed, format!("Query failed: {}", err))
    }

    pub fn queries_subscribe_failed<E: Display>(err: E) -> ApiError {
        error(ErrorCode::SubscribeFailed, format!("Subscribe failed: {}", err))
    }

    pub fn queries_wait_for_failed<E: Display>(err: E) -> ApiError {
        error(ErrorCode::WaitForFailed, format!("WaitFor failed: {}", err))
    }

    pub fn queries_get_subscription_result_failed<E: Display>(err: E) -> ApiError {
        error(
            ErrorCode::GetSubscriptionResultFailed,
            format!("Receive subscription result failed: {}", err),
        )
    }
}
