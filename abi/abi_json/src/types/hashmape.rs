use super::common::{
    find_next_bits,
    provide_empty_reference
};
use super::{
    ABISerialized,
    ABIDeserialized,
    DeserializationError
};

use tvm::stack::{BuilderData, SliceData, IBitstring};
use tvm::stack::dictionary::{HashmapE, HashmapType};

impl ABISerialized for HashmapE {
    fn prepend_to(&self, mut destination: BuilderData) -> BuilderData {
        match self.data() {
            Some(cell) => {
                destination = provide_empty_reference(destination);
                destination.prepend_bitstring(&[0b11000000]).unwrap();
                destination.prepend_reference(BuilderData::from(cell));
            }
            None => {
                destination.prepend_bitstring(&[0b01000000]).unwrap();
            }
        };

        destination
    }

    fn get_in_cell_size(&self) -> usize {
        1
    }
}

impl ABIDeserialized for HashmapE {
    type Out = SliceData;

    fn read_from(mut cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError> {
        let original = cursor.clone();
        cursor = find_next_bits(cursor, 1)?;
        match cursor.get_dictionary() {
            Ok(value) => Ok((value, cursor)),
            Err(_) => Err(DeserializationError::with(original))
        }
    }
}
