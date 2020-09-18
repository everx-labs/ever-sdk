use crate::error::ApiError;
const NET: isize = ApiError::NET; // 500

pub enum ErrorCode {
    Base = NET,
}

pub struct Error;

fn error(code: ErrorCode, message: String) -> ApiError {
    ApiError::with_code_message(code as isize, message)
}

impl Error {
}
