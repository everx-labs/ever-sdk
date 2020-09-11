use crate::error::ApiError;
use std::fmt::Display;

pub enum Code {
    QueriesQueryFailed = 600,
    QueriesSubscribeFailed = 601,
    QueriesWaitForFailed = 602,
    QueriesGetSubscriptionResultFailed = 603,
}
pub struct Error;

fn error(code: Code, message: String) -> ApiError {
    ApiError::with_code_message(code as isize, message)
}

impl Error {
    pub fn queries_query_failed<E: Display>(err: E) -> ApiError {
        error(
            Code::QueriesQueryFailed,
            format!("Query failed: {}", err))
    }

    pub fn queries_subscribe_failed<E: Display>(err: E) -> ApiError {
        error(
            Code::QueriesSubscribeFailed,
            format!("Subscribe failed: {}", err))
    }

    pub fn queries_wait_for_failed<E: Display>(err: E) -> ApiError {
        error(
            Code::QueriesWaitForFailed,
            format!("WaitFor failed: {}", err))
    }

    pub fn queries_get_subscription_result_failed<E: Display>(err: E) -> ApiError {
        error(
            Code::QueriesGetSubscriptionResultFailed,
            format!("Receive subscription result failed: {}", err))
    }
}
