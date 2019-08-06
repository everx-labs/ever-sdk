use super::common::*;
use super::common_arrays::*;
use super::{
    reader::Reader,
    ABISerialized,
    ABIDeserialized,
    DeserializationError,
    ABITypeSignature
};

use tvm::bitstring::{Bit, Bitstring};
use tvm::stack::{BuilderData, SliceData};

// put dynamic array to chain or to separate branch depending on array size
pub fn prepend_dynamic_array<T: ABISerialized>(
    mut destination: BuilderData,
    array: &[T],
) -> BuilderData {
    let mut array_size = 0;
    for i in array {
        array_size += i.get_in_cell_size();
    }

    // if array doesn't fit into one cell, we put into separate chain
    // Note: Since length is one byte value any array longer than 255
    // must be written into a separate cell.

    if array.len() > 255 || array_size > BuilderData::bits_capacity() {
        destination = put_array_to_separate_branch(destination, array);
    } else {
        // if array fit into cell data, put in into main chain
        destination = prepend_array_items_to_chain(destination, array);

        let mut bitstring = Bitstring::new();
        bitstring.append_bit(&Bit::One);
        bitstring.append_bit(&Bit::Zero);
        bitstring.append_u8(array.len() as u8);

        destination = prepend_data_to_chain(destination, bitstring);
    }

    destination
}

impl<T: ABISerialized> ABISerialized for Vec<T> {

    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        prepend_dynamic_array(destination, self.as_slice())
    }

    fn get_in_cell_size(&self) -> usize {
        let mut result = 8;
        for i in self {
            result += i.get_in_cell_size();
        }

        // if array doesn't fit into cell it is put in separate chain and only 2 bits are put in main chain cell
        if self.len() > 255 || result > BuilderData::bits_capacity() {
            2
        } else {
            result + 2
        }
    }
}

impl<T: ABIDeserialized> ABIDeserialized for Vec<T> {
    type Out = Vec<T::Out>;

    fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError> {
        let mut cursor = Reader::new(cursor);
        let flag = cursor.read_next::<(bool, bool)>()?;
        match flag {
            (false, false) => {
                let mut cursor = cursor.remainder();
                if cursor.remaining_references() == 0 {
                    return Err(DeserializationError::with(cursor));
                }
                let array = cursor.checked_drain_reference().unwrap();
                let mut array = Reader::new(array);
                let mut result = vec![];
                while !array.is_empty() {
                    result.push(array.read_next::<T>()?);
                }
                Ok((result, cursor))
            }
            (true, false) => {
                let size = cursor.read_next::<u8>()?;
                let mut result = vec![];
                for _ in 0..size {
                    result.push(cursor.read_next::<T>()?);
                }
                Ok((result, cursor.remainder()))
            }
            _ => Err(DeserializationError::with(cursor.remainder())),
        }
    }
}

impl<T: ABITypeSignature> ABITypeSignature for Vec<T> {
    fn type_signature() -> String {
        T::type_dynamic_array_signature()
    }
}
