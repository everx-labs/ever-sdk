use tonlabs_sdk_emulator::stack::BuilderData;

pub trait ABIParameter {
    fn prepend_to(&self, destination: BuilderData) -> BuilderData;
    fn type_signature() -> String;
}

pub mod common;

mod bool;
pub use self::bool::*;

mod tuples;
pub use self::tuples::*;

