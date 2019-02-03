use tonlabs_sdk_emulator::stack::{
    BuilderData,
    SliceData
};

#[derive(Debug)]
pub struct DeserializationError {
    pub cursor: SliceData
}

pub trait ABIParameter {
    // put data into chain
    fn prepend_to(&self, destination: BuilderData) -> BuilderData;

    // return type signature regarding to ABI specification
    fn type_signature() -> String;
    
    // return size in bits that are put into main chain during serialization 
    // (not whole parameter size - large arrays are put in separate chains and only 2 bits get into main chain)
    fn get_in_cell_size(&self) -> usize;

    fn read_from(cursor: SliceData) -> Result<(Self, SliceData), DeserializationError>
        where Self: std::marker::Sized;

    fn is_restricted_to_root() -> bool {
        return false;
    }
}

impl DeserializationError {
    pub fn with(cursor: SliceData) -> DeserializationError {
        DeserializationError {
            cursor
        }    
    }
}

pub mod reader;

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
