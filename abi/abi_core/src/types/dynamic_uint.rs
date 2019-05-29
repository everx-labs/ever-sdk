use super::dynamic_int::*;
use super::{
    ABISerialized,
    ABIDeserialized,
    ABITypeSignature,
    DeserializationError
};

use num_bigint::{BigUint, Sign};

use tvm::stack::{BuilderData, SliceData};

pub type Duint = BigUint;

impl ABISerialized for Duint {

    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        let dynamic_int = Dint::from_biguint(Sign::Plus, self.clone());

        dynamic_int.prepend_to(destination)
    }

    fn get_in_cell_size(&self) -> usize {
        let num_size = self.to_bytes_be().len() * 8;
        // split by groups of 7 bits with adding one bit to each group and last group pad to 8 bits
        num_size + num_size / 7 + ((num_size % 7) + 7) & !7
    }
}

impl ABIDeserialized for Duint {
    type Out = Duint;

    fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError> {
        let (vec, cursor) = read_dynamic_int(cursor, false)?;

        Ok((Duint::from_bytes_be(&vec), cursor))
    }
}

impl ABITypeSignature for Duint {
    fn type_signature() -> String {
        "duint".to_string()
    }
}
