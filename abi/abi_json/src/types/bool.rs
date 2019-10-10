use super::{
    ABISerialized,
    ABIDeserialized,
    DeserializationError
};
use super::common::find_next_bits;

use tvm::stack::{BuilderData, SliceData};

impl ABISerialized for bool {

    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        let mut destination = {
            if destination.bits_free() == 0 {
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

    fn read_from(mut cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError> {
        cursor = find_next_bits(cursor, 1)?;
        Ok((cursor.get_next_bit().unwrap(), cursor))
    }
}
