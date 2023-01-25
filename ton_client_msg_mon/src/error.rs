use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Error {
    pub code: u32,
    pub message: String,
    pub data: Value,
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
