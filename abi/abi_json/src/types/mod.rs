use tvm::stack::{BuilderData, SliceData};

#[derive(Debug)]
pub struct DeserializationError {
    pub cursor: SliceData,
}

/// Trait for values that can be serialized
pub trait ABISerialized {
    /// Puts data to the chain beginning
    fn prepend_to(&self, destination: BuilderData) -> BuilderData;

    /// Returns size in bits that are put into main chain during serialization
    /// (not whole parameter size - large arrays are put in separate chains and only 2 bits get into main chain)
    fn get_in_cell_size(&self) -> usize;
}

/// Trait for values that can be deserialized.
pub trait ABIDeserialized {
    /// Value type that current type are deserialized to
    type Out;

    /// Deserializes value from SliceData
    fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError>
    where
        Self::Out: std::marker::Sized;
}

impl DeserializationError {
    pub fn with(cursor: SliceData) -> DeserializationError {
        DeserializationError { cursor }
    }
}

pub mod reader;

mod bool;

#[macro_use]
mod common;
pub use self::common::*;

#[macro_use]
pub mod int;
pub use self::int::*;

mod bitstring;
pub use self::bitstring::*;

mod bit;
pub use self::bit::*;

mod hashmape;
pub use self::hashmape::*;
