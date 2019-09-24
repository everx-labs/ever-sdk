use super::common::{
    find_next_bit,
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
        destination = provide_empty_reference(destination);

        let mut slice = self.get_data();
        destination.prepend_builder(&BuilderData::from_slice(&slice)).unwrap();
        if let Ok(cell) = slice.checked_drain_reference() {
            destination.prepend_reference(BuilderData::from(cell));
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
        cursor = find_next_bit(cursor)?;

        let value = cursor
            .get_dictionary()
            .map_err(|_| DeserializationError { cursor: cursor.clone() })?;
        Ok((value, cursor))
    }
}
