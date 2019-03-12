use super::{
    ABIParameter,
    ABITypeSignature,
    DeserializationError
};

use tvm::bitstring::{Bit, Bitstring};
use tvm::stack::{BuilderData, SliceData};

impl ABIParameter for bool {
    type Out = bool;

    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        let mut destination = {
            if 1 + destination.bits_used() > destination.bits_capacity() {
                let mut next = BuilderData::new();
                next.append_reference(destination);
                next
            } else {
                destination
            }
        };
        destination.prepend_data(
            Bitstring::new().append_bit(&{
                if *self {
                    Bit::One
                } else {
                    Bit::Zero
                }
            }),
        );
        destination
    }

    fn get_in_cell_size(&self) -> usize {
        1
    }

    fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError> {
        let mut cursor = cursor;
        while cursor.remaining_bits() == 0 && cursor.remaining_references() == 1 {
            cursor = cursor.checked_drain_reference().unwrap();
        }
        if cursor.remaining_bits() > 0 {
            let value = cursor.get_next_bit();
            Ok((value, cursor))
        } else {
            Err(DeserializationError::with(cursor))
        }
    }
}

impl ABITypeSignature for bool {
    fn type_signature() -> String {
        "bool".to_string()
    }
}
