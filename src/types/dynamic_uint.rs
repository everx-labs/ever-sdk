use super::dynamic_int::*;
use super::{
    ABIParameter, 
    ABIOutParameter,
    DeserializationError
};

use std::fmt;

use num_bigint::{BigUint, BigInt, Sign};

use tonlabs_sdk_emulator::stack::{BuilderData, SliceData};

#[derive(PartialEq, Eq)]
pub struct Duint
{
    pub data: BigUint,
}

makeOutParameter!(Duint);

impl ABIParameter for Duint {
    type Out = Duint;

    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        let dynamic_int = Dint {
            data: BigInt::from_biguint(Sign::Plus, self.data.clone())
        };

        dynamic_int.prepend_to(destination)
    }

    fn type_signature() -> String {
        "duint".to_string()
    }

    fn get_in_cell_size(&self) -> usize {
        let num_size = self.data.to_bytes_be().len() * 8;
        // split by groups of 7 bits with adding one bit to each group and last group pad to 8 bits
        num_size + num_size / 7 + ((num_size % 7) + 7) & !7
    }

    fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError> {
        let (vec, cursor) = read_dynamic_int(cursor, false)?;

        Ok((Duint{data: BigUint::from_bytes_be(&vec)}, cursor))
    }
}

impl fmt::Debug for Duint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data.fmt(f)
    }
}