use serde_json::Value;
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Error {
    pub code: u32,
    pub message: String,
    pub data: Value,
}

impl Error {
    pub fn invalid_boc<E: Display>(err: E) -> Self {
        Self::with_code_message(201, format!("Invalid BOC: {}", err))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(code: u32, message: String, data: Value) -> Self {
        Self {
            code,
            message,
            data,
        }
    }

    pub fn with_code_message(code: u32, message: String) -> Self {
        Self {
            code,
            message,
            data: json!({}),
        }
    }
}
