use tvm::stack::{BuilderData, SliceData, IBitstring};
use super::DeserializationError;
use types::ABIDeserialized;

// put data to cell and make chain if data doesn't fit into cell
pub fn prepend_data_to_chain(mut builder: BuilderData, data: BuilderData) -> BuilderData {
    let mut data = data;

    while data.length_in_bits() > 0 {
        let remaining_bits = BuilderData::bits_capacity() - builder.bits_used();

        if remaining_bits > 0 {
            // data does not fit into cell - fill current cell and take remaining data
            let mut data: SliceData = data.clone().into();
            data.shrink_references(0..0);
            if remaining_bits < data.remaining_bits() {
                let mut cut = data.clone();
                cut.shrink_data(data.remaining_bits() - remaining_bits..data.remaining_bits());
                builder.prepend_builder(&BuilderData::from_slice(&cut)).unwrap();
            } else {
                // data fit into current cell - no data remaining
                builder.prepend_builder(&BuilderData::from_slice(&data)).unwrap();
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

pub fn get_next_byte_from_chain(cursor: SliceData) -> Result<(u8, SliceData), DeserializationError> {
    let mut cursor = cursor;
    
    if cursor.remaining_bits() >= 8 {
        Ok((cursor.get_next_byte(), cursor))
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

pub fn get_next_bits_from_chain(cursor: SliceData, bits: usize) -> Result<(BuilderData, SliceData), DeserializationError> {
    let mut cursor = cursor;
    
    if cursor.remaining_bits() >= bits {
        Ok((cursor.get_next_bitstring(bits), cursor))
    }
    else {
        while cursor.remaining_bits() == 0 && cursor.remaining_references() == 1 {
            cursor = cursor.checked_drain_reference().unwrap();
        }

        let remaining_bits = cursor.remaining_bits();

        if remaining_bits == 0 {
            return Err(DeserializationError::with(cursor));
        }

        if remaining_bits >= bits {
            Ok((cursor.get_next_bitstring(bits), cursor))
        } else {
            let mut result = cursor.get_next_bitstring(remaining_bits);

            let (remain, cursor) = get_next_bits_from_chain(cursor, bits - result.length_in_bits())?;

            result.append_builder(&remain);

            Ok((result, cursor))
        }
    }
}