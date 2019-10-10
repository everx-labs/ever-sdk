use tvm::stack::SliceData;

#[derive(Debug)]
pub struct DeserializationError {
    pub cursor: SliceData,
}

impl DeserializationError {
    pub fn with(cursor: SliceData) -> DeserializationError {
        DeserializationError { cursor }
    }
}

#[macro_use]
pub mod int;
pub use self::int::*;

mod bitstring;
pub use self::bitstring::*;
