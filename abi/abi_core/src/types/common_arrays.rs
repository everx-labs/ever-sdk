use tvm::stack::BuilderData;
use tvm::stack::dictionary::HashmapE;

use super::common::*;
use super::ABISerialized;
use types::{Bit, Bitstring};

// put array items to provided chain
pub fn prepend_array_items_to_chain<T: ABISerialized>(
    mut destination: BuilderData,
    array: &[T],
) -> BuilderData {
    for i in array.iter().rev() {
        destination = i.prepend_to(destination);
    }

    destination
}

// create separate branch for array, put array items data there, reference that branch from provided chain and add tag of separate branch
pub fn put_array_to_separate_branch<T: ABISerialized>(
    mut destination: BuilderData,
    array: &[T],
) -> BuilderData {
    let mut array_builder = BuilderData::new();
    array_builder = prepend_array_items_to_chain(array_builder, array);

    destination = provide_empty_reference(destination);

    destination.prepend_reference(array_builder);

    let mut bitstring = Bitstring::new();
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_bit(&Bit::Zero);

    destination = prepend_data_to_chain(destination, bitstring);

    destination
}

// creates dictionary with indexes of an array items as keys and items as values
// and prepends dictionary to cell
pub fn put_array_as_dictionary<T: ABISerialized>(
    mut destination: BuilderData,
    array: &[T],
) -> BuilderData {
    let mut map = HashmapE::with_bit_len(32);

    for i in 0..array.len() {
        let mut index = BuilderData::new();
        index = (i as u32).prepend_to(index);

        let mut data = BuilderData::new();
        data = array[i].prepend_to(data);

        map.set(index.into(), &data.into()).unwrap();
    }

    destination = map.prepend_to(destination);
    
    let mut bitstring = Bitstring::new();
    bitstring.append_bit(&Bit::Zero);
    bitstring.append_bit(&Bit::One);
    bitstring.append_u32(array.len() as u32);

    prepend_data_to_chain(destination, bitstring)
}
