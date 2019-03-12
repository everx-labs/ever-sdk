use super::{
    ABIParameter,
    DeserializationError,
    ABITypeSignature
};

use tvm::bitstring::{Bit, Bitstring};
use tvm::stack::{BuilderData, SliceData};

impl ABIParameter for Bitstring {
    type Out = Bitstring;

    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        self.bits(0 .. self.length_in_bits())
            .data.prepend_to(destination)
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

impl ABITypeSignature for Bitstring {
    fn type_signature() -> String {
        "bitstring".to_string()
    }
}
