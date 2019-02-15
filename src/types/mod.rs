use std::ops::Range;

use tonlabs_sdk_emulator::stack::{BuilderData, SliceData};
use tonlabs_sdk_emulator::bitstring::Bitstring;

#[derive(Debug)]
pub struct DeserializationError {
    pub cursor: SliceData,
}

// Note: The reason ABIInParameter is separate 
// from ABIParameter is that we want to have
// unique type () "empty tuple" that is only
// acceptable as a root object (in or out)
// and can't be used in compound types, ex.: ((),()).
pub trait ABIInParameter {
    // put data into chain
    fn prepend_to(&self, destination: BuilderData) -> BuilderData;

    // return type signature regarding to ABI specification
    fn type_signature() -> String;
}

// Note: Due to the limitations with fixed array
// constructors an assosiated type Out were added.
// Limitations:
// - [T; n], T must be Copy, Default
// - this adds too much complexity to Vec<T>
pub trait ABIOutParameter {
    type Out;
 
    fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError>
    where
        Self::Out: std::marker::Sized;
}

pub trait ABIParameter {
    type Out;

    // put data into chain
    fn prepend_to(&self, destination: BuilderData) -> BuilderData;

    // return type signature regarding to ABI specification
    fn type_signature() -> String;

    // return size in bits that are put into main chain during serialization
    // (not whole parameter size - large arrays are put in separate chains and only 2 bits get into main chain)
    fn get_in_cell_size(&self) -> usize;

    fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError>
    where
        Self::Out: std::marker::Sized;
}

impl DeserializationError {
    pub fn with(cursor: SliceData) -> DeserializationError {
        DeserializationError { cursor }
    }
}


pub trait SubString {
    fn substring(&self, range: Range<usize>) -> Bitstring;
}

impl SubString for Bitstring {
    fn substring(&self, range: Range<usize>) -> Bitstring {
        let mut result = Bitstring::new();

        self.bits(range)
            .data
            .iter()
            .for_each(|x| {
                result.append_bit(x);
        });

        result
    }
}

pub mod reader;

#[macro_use]
pub mod common;
pub mod common_arrays;

mod bool;
pub use self::bool::*;

mod int;
pub use self::int::*;

mod tuples;
pub use self::tuples::*;

mod fixed_array;
pub use self::fixed_array::*;

mod dynamic_array;
pub use self::dynamic_array::*;

mod bitstring;
pub use self::bitstring::*;

mod bit;
pub use self::bit::*;

mod bits;
pub use self::bits::*;

pub mod dynamic_int;
pub use self::dynamic_int::*;

pub mod dynamic_uint;
pub use self::dynamic_uint::*;