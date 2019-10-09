use tvm::stack::{BuilderData, SliceData, IBitstring};
use super::DeserializationError;
use types::{ABIDeserialized, Bitstring};

// put data to cell and make chain if data doesn't fit into cell
pub fn prepend_data_to_chain(mut builder: BuilderData, data: Bitstring) -> BuilderData {
    let mut data = data;

    while data.length_in_bits() > 0 {
        let remaining_bits = builder.bits_free();

        if remaining_bits > 0 {
            // data does not fit into cell - fill current cell and take remaining data
            if remaining_bits < data.length_in_bits() {
                let cut = data.substring(data.length_in_bits() - remaining_bits..data.length_in_bits());
                let mut vec = vec![];
                cut.into_bitstring_with_completion_tag(&mut vec);
                builder.prepend_bitstring(&vec).unwrap();

                data = data.substring(0..data.length_in_bits() - remaining_bits);
            } else {
                // data fit into current cell - no data remaining
                let mut vec = vec![];
                data.into_bitstring_with_completion_tag(&mut vec);
                builder.prepend_bitstring(&vec).unwrap();

                data.clear();
            }
        } else {
            // current cell is full - move to next
            let mut next_builder = BuilderData::new();
            next_builder.append_reference(builder);
            builder = next_builder;
        }
    }

    builder
}

macro_rules! next_byte {
    ( $cursor:ident ) => {
        $cursor.get_next_byte().map_err(|_| DeserializationError { cursor: $cursor.clone() })?
    }
}

pub fn get_next_byte_from_chain(
    cursor: SliceData
) -> Result<(u8, SliceData), DeserializationError> {
    let mut cursor = cursor;    
    if cursor.remaining_bits() >= 8 {
        Ok((next_byte!(cursor), cursor))
    }
    else {
        let mut result: u8 = 0;
        for i in (0..8).rev() {
            let (bit, new_cursor) = <bool as ABIDeserialized>::read_from(cursor)?;
            cursor = new_cursor;
            if bit {
                result |= 1 << i;
            }
        }
        Ok((result, cursor))
    }
}

pub fn find_next_bits(mut cursor: SliceData, bits: usize) -> Result<SliceData, DeserializationError> {
    let original = cursor.clone();
    if cursor.remaining_bits() == 0 {
        cursor = cursor.reference(0)
            .map(|cell| cell.into())
            .map_err(|_| DeserializationError::with(cursor))?;
    }
    match cursor.remaining_bits() >= bits  {
        true => Ok(cursor),
        false => Err(DeserializationError::with(original))
    }
}

pub fn get_next_bits_from_chain(mut cursor: SliceData, bits: usize)
-> Result<(Vec<u8>, SliceData), DeserializationError> {
    cursor = find_next_bits(cursor, bits)?;
    Ok((cursor.get_next_bits(bits).unwrap(), cursor))
}

// if currnet cell is filled with references (one reference is reserved for chaining cells) or data,
// then we append reference to next cell
pub fn provide_empty_reference(destination: BuilderData) -> BuilderData {
    if destination.references_free() == 0 || destination.bits_free() == 0 {
        let mut next = BuilderData::new();
        next.append_reference(destination);
        next
    } else {
        destination
    }
}
