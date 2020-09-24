use crate::error::ApiError;
use crate::net::process_message::TransactionWaitingState;
use serde_json::Value;

const NET: isize = ApiError::NET; // 500

pub enum ErrorCode {
    MessageAlreadyExpired = NET + 1,
    MessageHasNotDestinationAddress = NET + 2,
}

pub struct Error;

fn error(code: ErrorCode, message: String) -> ApiError {
    ApiError::with_code_message(code as isize, message)
}

fn error_with_data(code: ErrorCode, message: String, data: Value) -> ApiError {
    ApiError::with_code_message_data(code as isize, message, data)
}

impl Error {
    pub fn message_already_expired() -> ApiError {
        error(
            ErrorCode::MessageAlreadyExpired,
            "Message can’t be sent because it is expired".into(),
        )
    }

    pub fn message_has_not_destination_address() -> ApiError {
        error(
            ErrorCode::MessageHasNotDestinationAddress,
            "Message can't be sent because it hasn't destination address".into(),
        )
    }

    pub fn fetch_block_failed(
        message_id: &str,
        waiting_state: &TransactionWaitingState,
        timeout: u32,
    ) -> ApiError {
        let block_id = waiting_state
            .expiration
            .as_ref()
            .map(|x| x.last_checked_block_id);
        error_with_data(
            ErrorCode::fetch_block_failed,
            "".into(),
            json!({
                "message_id": message_id,
                "message_sending_time": waiting_state.message_senging_time,
                "last_checked_block_id": block_id,
                "timeout": timeout,
            }),
        )
    }
}
