use tonlabs_sdk_emulator::stack::BuilderData;

pub trait ABIParameter {
    fn prepend_to(&self, destination: BuilderData) -> BuilderData;
    fn type_signature() -> String;
}

pub mod common;

mod bool;
pub use self::bool::*;

mod int;
pub use self::int::*;

mod tuples;
pub use self::tuples::*;

mod array;
pub use self::array::*;

mod slice;
pub use self::slice::*;

mod vec;
pub use self::vec::*;