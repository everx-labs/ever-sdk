use crate::error::ApiError;
const NET: isize = ApiError::NET; // 500

pub enum ErrorCode {
    MessageAlreadyExpired = NET + 1,
    MessageHasNotDestinationAddress = NET + 2,
}

pub struct Error;

fn error(code: ErrorCode, message: String) -> ApiError {
    ApiError::with_code_message(code as isize, message)
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
}
