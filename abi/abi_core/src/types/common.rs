use tvm::stack::{BuilderData, SliceData, IBitstring};
use super::DeserializationError;
use types::ABIDeserialized;

// put data to cell and make chain if data doesn't fit into cell
pub fn prepend_data_to_chain(mut builder: BuilderData, data: BuilderData) -> BuilderData {
    let mut data: SliceData = data.into();
    data.shrink_references(0..0);
    while data.remaining_bits() > 0 {
        let remaining_bits = BuilderData::bits_capacity() - builder.bits_used();
        if remaining_bits > 0 {
            let data_bits = data.remaining_bits();
            if remaining_bits < data_bits {
                // data does not fit into cell - fill current cell and take remaining data
                let mut cut = data.clone();
                cut.shrink_data(data_bits - remaining_bits..data_bits);
                builder.prepend_builder(&BuilderData::from_slice(&cut)).unwrap();
                data.shrink_data(0..data_bits - remaining_bits);
            } else {
                // data fit into current cell - no data remaining
                builder.prepend_builder(&BuilderData::from_slice(&data)).unwrap();
                data.shrink_data(0..0);
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

macro_rules! next_slice {
    ( $cursor:ident, $bits:ident ) => {
        $cursor
            .get_next_slice($bits)
            .map_err(|_| DeserializationError { cursor: $cursor.clone() })?
    }
}

pub fn get_next_bits_from_chain(
    cursor: SliceData, 
    bits: usize
) -> Result<(SliceData, SliceData), DeserializationError> {
    let mut cursor = cursor;    
    if cursor.remaining_bits() >= bits {
        Ok((next_slice!(cursor, bits), cursor))
    }
    else {
        while (cursor.remaining_bits() == 0) && (cursor.remaining_references() == 1) {
            cursor = cursor.checked_drain_reference().unwrap();
        }
        let remaining_bits = cursor.remaining_bits();
        if remaining_bits == 0 {
            return Err(DeserializationError::with(cursor));
        }
        if remaining_bits >= bits {
            Ok((next_slice!(cursor, bits), cursor))
        } else {
            let result = next_slice!(cursor, remaining_bits);
            let (remain, cursor) = get_next_bits_from_chain(
                cursor, 
                bits - result.remaining_bits()
            )?;
            let mut builder = BuilderData::from_slice(&result);
            builder
                .checked_append_references_and_data(&remain)
                .map_err(|_| DeserializationError { cursor: cursor.clone() })?;
            Ok((builder.into(), cursor))
        }
    }
}
