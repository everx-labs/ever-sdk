use tvm::stack::{BuilderData, SliceData};

#[derive(Debug)]
pub struct DeserializationError {
    pub cursor: SliceData,
}


/// Trait for values that can be passed to `ABICall::encode_function_call` and
/// `ABICall::encode_function_call_into_slice` functions
pub trait ABIInParameter {
    // Note: The reason ABIInParameter is separate 
    // from ABIParameter is that we want to have
    // unique type () "empty tuple" that is only
    // acceptable as a root object (in or out)
    // and can't be used in compound types, ex.: ((),()).

    /// Puts data to the chain beginning
    fn prepend_to(&self, destination: BuilderData) -> BuilderData;
}

/// Trait for values that can be produced by `ABIResponse::decode_response` and
/// `ABIResponse::decode_response_from_slice` functions
pub trait ABIOutParameter {
    // Note: Due to the limitations with fixed array
    // constructors an assosiated type Out were added.
    // Limitations:
    // - [T; n], T must be Copy, Default
    // - this adds too much complexity to Vec<T>

    /// Value type that current type are deserialized to
    type Out;
 
    /// Deserializes value from `SliceData`
    fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError>
    where
        Self::Out: std::marker::Sized;
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

/// Trait providing type names for function signature composing
pub trait ABITypeSignature {

    /// Returns type name regarding to ABI specification
    fn type_signature() -> String;

    /// Returns type name of fixed array of elements regarding to ABI specification
    fn type_fixed_array_signature(size: usize) -> String {
        format!("{}[{}]", Self::type_signature(), size)
    }

    /// Returns type name of dynamic array of elements regarding to ABI specification
    fn type_dynamic_array_signature() -> String {
        format!("{}[]", Self::type_signature())
    }
}

impl DeserializationError {
    pub fn with(cursor: SliceData) -> DeserializationError {
        DeserializationError { cursor }
    }
}

pub mod reader;

#[macro_use]
mod common;
pub use self::common::*;

pub mod common_arrays;

mod bool;
pub use self::bool::*;

mod int;
pub use self::int::*;

mod tuples;
pub use self::tuples::*;

#[macro_use]
mod fixed_array;
pub use self::fixed_array::*;

mod dynamic_array;
pub use self::dynamic_array::*;

mod bitstring;
pub use self::bitstring::*;

mod bit;
pub use self::bit::*;

mod hashmape;
pub use self::hashmape::*;
