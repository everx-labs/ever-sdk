use tonlabs_sdk_emulator::stack::BuilderData;
use tonlabs_sdk_emulator::bitstring::{Bit, Bitstring};
use types::ABIParameter;

pub fn prepend_reference(builder: &mut BuilderData, child: BuilderData) {
    builder.update_cell(|_, children, child| {
        children.insert(0, child);
    }, child);
}

// shifts existing cell data and put provided data at the beginning
pub fn prepend_data(builder: &mut BuilderData, data: &Bitstring) {
    builder.update_cell(|cell_data, _, data| {
        let mut buffer = data.clone();
        buffer.append(
            &Bitstring::from_bitstring_with_completion_tag(cell_data.clone())
        );
        cell_data.clear();
        buffer.into_bitstring_with_completion_tag(cell_data);
    }, data);
}

// put data to cell and make chain if data doesn't fit into cell
pub fn prepend_data_to_chain(mut builder: BuilderData, data: Bitstring) -> BuilderData {
    let mut data = data;

    while data.length_in_bits() > 0 {
        let remaining_bits = builder.bits_capacity() - builder.bits_used();

        if remaining_bits > 0 {
            // data does not fit into cell - fill current cell and take remaining data 
            if remaining_bits < data.length_in_bits(){
                let mut cut = Bitstring::new();
                // TODO: replace iteration on Bits with Bitstring::substring function
                data.bits(data.length_in_bits() - remaining_bits .. data.length_in_bits()).data.iter().for_each(|x| { cut.append_bit(x); });
                prepend_data(&mut builder, &cut);

                cut.clear();
                data.bits(0 .. data.length_in_bits() - remaining_bits).data.iter().for_each(|x| { cut.append_bit(x); });

                data = cut;
            }
            else{
                // data fit into current cell - no data remaining
                prepend_data(&mut builder, &data);

                data.clear();
            }
        }
        else{
            // current cell is full - move to next
            let mut next_builder = BuilderData::new();
            next_builder.append_reference(builder);
            builder = next_builder;
        }
    }

    builder 
}

// put array data to chain or to separate chain depending on array size
pub fn prepend_array<T: ABIParameter>(destination: BuilderData, array: &[T], set_length: bool) -> BuilderData {
    let mut bitstring = Bitstring::new();
    let mut destination = destination;

    // if array doesn't fit into one cell, we put into separate chain
    if (array.len() * std::mem::size_of::<T>() * 8) > destination.bits_capacity() {
        let mut array_builder = BuilderData::new();
        for i in array.iter().rev() {
            array_builder = i.prepend_to(array_builder);
        }

        // dynamic arrays are not prepended by length when put in separate chain

        // if currnet cell is filled with references (one teference is reserved for chaining cells) or data,
        // then we append reference to next cell
        destination = {
            if  destination.references_used() == destination.references_capacity() ||
                destination.bits_used() == destination.bits_capacity() {
                let mut next = BuilderData::new();
                next.append_reference(destination);
                next
            } else {
                destination
            }
        };

        prepend_reference(&mut destination, array_builder);

        bitstring.append_bit(&Bit::Zero);
        bitstring.append_bit(&Bit::Zero);
    }
    else {
        // if array fit into cell data, put in into main chain
        for i in array.iter().rev() {
            destination = i.prepend_to(destination);
        }

        bitstring.append_bit(&Bit::One);
        bitstring.append_bit(&Bit::Zero);

        if set_length {
           bitstring.append_u8(array.len() as u8);
        }
    }

    prepend_data_to_chain(destination, bitstring)
}