use tvm::stack::{BuilderData, IBitstring};

use super::common::*;
use super::ABISerialized;

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

    // if currnet cell is filled with references (one reference is reserved for chaining cells) or data,
    // then we append reference to next cell
    destination = {
        if destination.references_used() == BuilderData::references_capacity()
            || destination.bits_used() == BuilderData::bits_capacity()
        {
            let mut next = BuilderData::new();
            next.append_reference(destination);
            next
        } else {
            destination
        }
    };

    destination.prepend_reference(array_builder);

    let mut bitstring = BuilderData::new();
    bitstring.append_bit_zero().unwrap();
    bitstring.append_bit_zero().unwrap();

    destination = prepend_data_to_chain(destination, bitstring);

    destination
}
