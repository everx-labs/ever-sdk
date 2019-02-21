use super::common::*;
use super::{
    ABIParameter,
    ABITypeSignature,
    DeserializationError
};

use num_bigint::{BigInt, Sign};

use tvm::bitstring::{Bit, Bitstring};
use tvm::stack::{BuilderData, SliceData};

pub fn read_dynamic_int(cursor: SliceData, signed_padding: bool) -> Result<(Vec<u8>, SliceData), DeserializationError> {
    let mut cursor = cursor;
    let mut bitstring = Bitstring::new();

    loop {
        let (byte, new_cursor) = <u8 as ABIParameter>::read_from(cursor)?;
        cursor = new_cursor;

        bitstring = Bitstring::create(vec![byte << 1], 7) + bitstring;

        if (byte & 0x80) == 0 {
            break;
        }
    };

    // pad to 8 bits
    let padding_count = 8 - ((bitstring.length_in_bits() - 1) % 8 + 1);

    let padding: u16 =
        if signed_padding && (bitstring.bits(0..1).data[0] == Bit::One) {
            0xFFFF
        } else {
            0x0
        };

    let mut padding_string = Bitstring::new();
    padding_string.append_bits(padding, padding_count);

    bitstring = padding_string + bitstring;

    let mut vec = Vec::<u8>::new();
    bitstring.into_bitstring_with_completion_tag(&mut vec);
    vec.pop();

    Ok((vec, cursor))
}

pub type Dint = BigInt;

impl ABIParameter for Dint {
    type Out = Dint;

    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
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

        let num_bitstring = Bitstring::create(bytes, size);

        let cropped_bitstring = num_bitstring.substring(crop_bits..num_bitstring.length_in_bits());

        let mut result = Bitstring::new();
        let mut remain = cropped_bitstring.length_in_bits();

        // take gropus by 7 bits
        while remain > 0 {
            let bit_count = std::cmp::min(remain, 7);

            let mut prefix = Bitstring::new();

            // add prefix (1 - more groups followed, 0 - last group)
            if remain > bit_count {
                prefix.append_bit(&Bit::One);
            } else {
                prefix.append_bit(&Bit::Zero);
                if bit_count != 7 {
                    // pad last group to 7 bits according to number sign
                    let padding: u16 = match self.sign() {
                        Sign::Plus => 0x0,
                        Sign::NoSign => 0x0,
                        Sign::Minus => 0xffff,
                    };
                    prefix.append_bits(padding, 7 - bit_count);
                }
            }

            result = result + prefix + cropped_bitstring.substring(remain - bit_count .. remain);

            remain -= bit_count;
        }

        prepend_data_to_chain(destination, result)
    }

    fn get_in_cell_size(&self) -> usize {
        let num_size = self.to_signed_bytes_be().len() * 8;
        // split by groups of 7 bits with adding one bit to each group and last group pad to 8 bits
        num_size + num_size / 7 + ((num_size % 7) + 7) & !7
    }

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
