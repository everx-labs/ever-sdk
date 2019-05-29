use super::{
    ABISerialized,
    ABIDeserialized,
    ABITypeSignature,
    DeserializationError
};

use tvm::stack::{BuilderData, SliceData};

impl ABISerialized for bool {

    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        let mut destination = {
            if 1 + destination.bits_used() > BuilderData::bits_capacity() {
                let mut next = BuilderData::new();
                next.append_reference(destination);
                next
            } else {
                destination
            }
        };

        let vec = if *self {
            [0x80]
        } else {
            [0x00]
        };
        destination.prepend_raw(&vec, 1).unwrap();
        destination
    }

    fn get_in_cell_size(&self) -> usize {
        1
    }
}

impl ABIDeserialized for bool {
    type Out = bool;

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
