use super::common::prepend_data;
use super::{
    ABIParameter,
    DeserializationError
};

use tonlabs_sdk_emulator::bitstring::{Bit, Bitstring};
use tonlabs_sdk_emulator::stack::{
    BuilderData,
    SliceData
};

impl ABIParameter for bool {
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
        prepend_data(
            &mut destination,
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

    fn type_signature() -> String {
        "bool".to_string()
    }

    fn get_in_cell_size(&self) -> usize {
        1
    }

    fn read_from(cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
        let mut cursor = cursor;
        while cursor.remaining_bits() == 0 && cursor.remaining_references() == 1 {
            cursor = cursor.drain_reference();
        }
        if cursor.remaining_bits() > 0 {
            let value = cursor.get_next_bit();
            Ok((value, cursor))
        } else {
            Err(DeserializationError::with(cursor))
        }
    }
}
