use super::common::*;
use super::{
    ABISerialized,
    ABIDeserialized,
    ABITypeSignature,
    DeserializationError
};

use num_bigint::{BigInt, Sign};
use tvm::stack::{BuilderData, IBitstring, SliceData};

pub fn read_dynamic_int(
    cursor: SliceData, 
    signed_padding: bool
) -> Result<(Vec<u8>, SliceData), DeserializationError> {
    let mut cursor = cursor;
    let mut bitstring = BuilderData::new();
    loop {
        let (byte, new_cursor) = <u8 as ABIDeserialized>::read_from(cursor)?;
        cursor = new_cursor;
        bitstring
            .prepend_raw(&[byte << 1], 7)
            .map_err(|_| DeserializationError{ cursor: cursor.clone() })?;
        if (byte & 0x80) == 0 {
            break;
        }
    };
    Ok((bitstring_to_be_bytes(bitstring.into(), signed_padding)?, cursor))
}

pub fn bitstring_to_be_bytes(
    bitstring: SliceData, 
    signed_padding: bool
) -> Result<Vec<u8>, DeserializationError> {
    let total_bits = bitstring.remaining_bits();
    let padding = 8 - total_bits % 8;
    if padding < 8 {    
        let mut ret = Vec::new();
        let mut bits = 8 - padding;
        let mut byte = bitstring
            .get_bits(0, bits)
            .map_err(|_| DeserializationError { cursor: bitstring.clone() })?;
        if signed_padding && (byte & (1 << (bits - 1)) != 0) {
            byte |= 0xFFu8 << bits;
        }
        ret.push(byte);
        while bits < total_bits {
            ret.push(
                bitstring
                    .get_bits(bits, 8)
                    .map_err(|_| DeserializationError { cursor: bitstring.clone() })?
            );
            bits += 8;
        }
        Ok(ret)
    } else {
        Ok(bitstring.cell().data().to_vec())
    }
}

pub type Dint = BigInt;

impl ABISerialized for Dint {

    fn prepend_to(&self, destination: BuilderData) -> BuilderData {

        let bytes = self.to_signed_bytes_be();
        let padding = match self.sign() {
            Sign::Minus => 0xFFu8,
            _ => 0x00u8
        };

        // Skip unsignificant high bits 
        let skip_bits = std::cmp::min((bytes[0] ^ padding).leading_zeros(), 7) as usize;
        let mut c = bytes.len() * 8 - 1;
        let mut s = 0usize;
        let mut b = BuilderData::new();

        while c >= skip_bits {
            let mut byte = bytes[c / 8];
            if s > 0 {
                byte >>= s;
                if c > 8 {
                    byte |= bytes[c / 8 - 1] << (8 - s);
                }
            }
            if c >= 8 + skip_bits {
                // Serialize as non-final byte
                byte |= 0x80;
            } else {
                // Serialize as final byte with padding
                if padding != 0 {
                    (7 + skip_bits).checked_sub(c).map(|shift|
                        byte |= padding << shift
                    );
                    byte &= 0x7F;
                }
            }
            b.append_u8(byte).unwrap();
            c = c.checked_sub(7).unwrap_or(0);
            s = s.checked_sub(1).unwrap_or(7);
        }

        prepend_data_to_chain(destination, b)

/*
        let bytes = self.to_signed_bytes_be();
        let size = bytes.len() * 8;

        let high_byte = bytes[0];
        let mut crop_bits = 0;

        // crop unsignificant high bits to reduce size
        for i in (0..7).rev() {
            if (high_byte >> 7) == (high_byte >> i & 0x01) {
                crop_bits += 1;
            } else {
                break;
            }
        }

        let cropped_bitstring = SliceData::from_raw(bytes, size);
        cropped_bitstring.shrink_data(crop_bits..size);

        let mut result = BuilderData::new();
        let mut remain = cropped_bitstring.remaining_bits();

        // take groups by 7 bits
        while remain > 0 {

            let bit_count = std::cmp::min(remain, 7);
            let mut prefix = 0u8;

            // add prefix (1 - more groups followed, 0 - last group)
            if remain > bit_count {
                prefix.append_bit_one().unwrap();
            } else {
                prefix.append_bit_zero().unwrap();
                if bit_count != 7 {
                    // pad last group to 7 bits according to number sign
                    let padding = match self.sign() {
                        Sign::Plus => 0x0,
                        Sign::NoSign => 0x0,
                        Sign::Minus => 0xffff,
                    };
                    prefix.append_bits(padding, 7 - bit_count).unwrap();
                }
            }

            result = result + prefix + cropped_bitstring.substring(remain - bit_count .. remain);

            remain -= bit_count;
        }

        prepend_data_to_chain(destination, result)
*/
    }

    fn get_in_cell_size(&self) -> usize {
        let num_size = self.to_signed_bytes_be().len() * 8;
        // split by groups of 7 bits with adding one bit to each group and last group pad to 8 bits
        num_size + num_size / 7 + ((num_size % 7) + 7) & !7
    }
}

impl ABIDeserialized for Dint {
    type Out = Dint;

    fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError> {
        let (vec, cursor) = read_dynamic_int(cursor, true)?;

        Ok((Dint::from_signed_bytes_be(&vec), cursor))
    }
}

impl ABITypeSignature for Dint {
    fn type_signature() -> String {
        "dint".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_dynamic_int() {
        let mut cell = BuilderData::new();
        cell.update_cell(|data, _, _, _, vec| {
            data.clear();
            data.extend_from_slice(vec);
        }, &[0xF8, 0xFF, 0x7F, 0xF8, 0xFF, 0x7F, 0x80]);
        dbg!(cell.clone());
        let cursor = cell.into();
        // check signed
        let (vec, new_cursor) = read_dynamic_int(cursor, true).unwrap();
        assert_eq!(vec, vec![0xFF, 0xFF, 0xF8]);
        assert_eq!(Dint::from_signed_bytes_be(&vec), Dint::from(-8));

        // check unsigned
        let (vec, _new_cursor) = read_dynamic_int(new_cursor, false).unwrap();
        assert_eq!(vec, vec![0x1F, 0xFF, 0xF8]);
        assert_eq!(Dint::from_signed_bytes_be(&vec), Dint::from(0x1FFFF8));
    }
}