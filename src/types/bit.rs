use super::{
    ABIParameter,
    DeserializationError,
    ABIOutParameter
};

use tonlabs_sdk_emulator::bitstring::Bit;
use tonlabs_sdk_emulator::stack::{BuilderData, SliceData};

makeOutParameter!(Bit);

impl ABIParameter for Bit {
    type Out = Bit;

    fn get_in_cell_size(&self) -> usize {
        true.get_in_cell_size()
    }
    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        (*self == Bit::One).prepend_to(destination)
    }

    fn type_signature() -> String {
        bool::type_signature()
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