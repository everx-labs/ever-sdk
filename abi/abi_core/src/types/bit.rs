use super::{
    ABISerialized,
    ABIDeserialized,
    DeserializationError,
    ABITypeSignature
};

use tvm::stack::{BuilderData, SliceData};

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Bit {
    Zero,
    One,
}

impl From<bool> for Bit {
    fn from(b: bool) -> Bit {
        if b {
            Bit::One
        } else {
            Bit::Zero
        }
    }
}

pub struct Bits {
    pub data: Vec<Bit>,
}

impl ABISerialized for Bit {
    fn get_in_cell_size(&self) -> usize {
        true.get_in_cell_size()
    }
    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        (*self == Bit::One).prepend_to(destination)
    }
}

impl ABIDeserialized for Bit {
    type Out = Bit;
 
    fn read_from(cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
        let (value, cursor) = <bool as ABIDeserialized>::read_from(cursor)?;

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