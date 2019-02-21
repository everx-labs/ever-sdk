use super::{
    ABIParameter,
    DeserializationError,
    ABITypeSignature
};

use tvm::bitstring::Bit;
use tvm::stack::{BuilderData, SliceData};

impl ABIParameter for Bit {
    type Out = Bit;

    fn get_in_cell_size(&self) -> usize {
        true.get_in_cell_size()
    }
    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        (*self == Bit::One).prepend_to(destination)
    }
 
    fn read_from(cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
        let (value, cursor) = <bool as ABIParameter>::read_from(cursor)?;

        let bit_value = if value {
            Bit::One
        } else {
            Bit::Zero

        };
        
        Ok((bit_value, cursor))
    }
}

impl ABITypeSignature for Bit {
    fn type_signature() -> String {
        bool::type_signature()
    }

    fn type_fixed_array_signature(size: usize) -> String {
        format!("bits{}", size)
    }

    fn type_dynamic_array_signature() -> String {
        format!("bitstring")
    }
}