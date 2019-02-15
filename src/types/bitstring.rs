use super::{
    ABIParameter,
    DeserializationError,
    ABIOutParameter
};

use tonlabs_sdk_emulator::bitstring::{Bit, Bitstring};
use tonlabs_sdk_emulator::stack::{BuilderData, SliceData};

makeOutParameter!(Bitstring);

impl ABIParameter for Bitstring {
    type Out = Bitstring;

    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        self.bits(0 .. self.length_in_bits())
            .data.prepend_to(destination)
    }

    fn type_signature() -> String {
        "bitstring".to_string()
    }

    fn get_in_cell_size(&self) -> usize {
        self.bits(0 .. self.length_in_bits())
            .data.get_in_cell_size()
    }

    fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError> {
        let (bits, cursor) = <Vec<Bit> as ABIParameter>::read_from(cursor)?;
        
        let mut result = Bitstring::new();
        bits.iter()
            .for_each(|x| {
                result.append_bit(x);
        });

        Ok((result, cursor))
    }
}
