use token::{TokenizeError, DetokenizeError};
use function::{SerializationError, DeserializationError};

#[derive(Debug)]
pub enum ABIError {
    SerdeError(serde_json::Error),
    InvalidName(String),
    InvalidFunctionId(u32),
    TokenizeError(TokenizeError),
    SerializationError(SerializationError),
    DeserializationError(DeserializationError),
    DetokenizeError(DetokenizeError),
    NotImplemented,
}
