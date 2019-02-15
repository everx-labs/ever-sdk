use super::common::*;
use super::{
    ABIParameter, 
    ABIOutParameter,
    DeserializationError,
    SubString
};

use std::fmt;

use num_bigint::{BigInt, Sign};

use tonlabs_sdk_emulator::bitstring::{Bit, Bitstring};
use tonlabs_sdk_emulator::stack::{BuilderData, SliceData};

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

#[derive(PartialEq, Eq)]
pub struct Dint
{
    pub  data: BigInt,
}

makeOutParameter!(Dint);

impl ABIParameter for Dint {
    type Out = Dint;

    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        let bytes = self.data.to_signed_bytes_be();
        let size = bytes.len() * 8;

        let num_bitstring = Bitstring::create(bytes, size);

        let mut result = Bitstring::new();
        let mut remain = num_bitstring.length_in_bits();

        // take gropus by 7 bits
        while remain > 0 {
            let bit_count = std::cmp::min(remain, 7);

            let mut prefix = Bitstring::new();

            // add prefix (1 - more groups followed, 0 - last group)
            if remain > bit_count {
                prefix.append_bit(&Bit::One);
            } else {
                prefix.append_bit(&Bit::Zero);
                // pad last group to 7 bits according to number sign
                let padding: u16 = match self.data.sign() {
                    Sign::Plus => 0x0,
                    Sign::NoSign => 0x0,
                    Sign::Minus => 0xffff,
                };
                prefix.append_bits(padding, 7 - bit_count);
            }

            result = result + prefix + num_bitstring.substring(remain - bit_count .. remain);

            remain -= bit_count;
        }

        prepend_data_to_chain(destination, result)
    }

    fn type_signature() -> String {
        "dint".to_string()
    }

    fn get_in_cell_size(&self) -> usize {
        let num_size = self.data.to_signed_bytes_be().len() * 8;
        // split by groups of 7 bits with adding one bit to each group and last group pad to 8 bits
        num_size + num_size / 7 + ((num_size % 7) + 7) & !7
    }

    fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError> {
        let (vec, cursor) = read_dynamic_int(cursor, true)?;

        Ok((Dint{data: BigInt::from_signed_bytes_be(&vec)}, cursor))
    }
}

impl fmt::Debug for Dint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data.fmt(f)
    }
}