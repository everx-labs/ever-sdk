use tvm::stack::{BuilderData, SliceData, IBitstring};
use super::DeserializationError;
use types::{ABIDeserialized, Bitstring};

// put data to cell and make chain if data doesn't fit into cell
pub fn prepend_data_to_chain(mut builder: BuilderData, data: Bitstring) -> BuilderData {
    let mut data = data;

    while data.length_in_bits() > 0 {
        let remaining_bits = BuilderData::bits_capacity() - builder.bits_used();

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

macro_rules! next_bitstring {
    ( $cursor:ident, $bits:ident ) => {
        $cursor
            .get_next_slice($bits)
            .map(|slice| Bitstring::create(slice.get_bytestring(0), $bits))
            .map_err(|_| DeserializationError { cursor: $cursor.clone() })?
    }
}

pub fn get_next_bits_from_chain(
    cursor: SliceData, 
    bits: usize
) -> Result<(Bitstring, SliceData), DeserializationError> {
    let mut cursor = cursor;    
    if cursor.remaining_bits() >= bits {
        Ok((next_bitstring!(cursor, bits), cursor))
    }
    else {
        while (cursor.remaining_bits() == 0) && (cursor.remaining_references() == 1) {
            cursor = cursor.checked_drain_reference().unwrap().into();
        }
        let remaining_bits = cursor.remaining_bits();
        if remaining_bits == 0 {
            return Err(DeserializationError::with(cursor));
        }
        if remaining_bits >= bits {
            Ok((next_bitstring!(cursor, bits), cursor))
        } else {
            let mut result = next_bitstring!(cursor, remaining_bits);
            let (remain, cursor) = get_next_bits_from_chain(
                cursor, 
                bits - result.length_in_bits()
            )?;

            result.append(&remain);
            Ok((result, cursor))
        }
    }
}
