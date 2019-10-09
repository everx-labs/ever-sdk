use num_bigint::{BigInt, Sign};
use tvm::stack::{BuilderData, SliceData};
use types::{
    ABIDeserialized,
    ABISerialized,
    ABITypeSignature,
    Bitstring,
    prepend_data_to_chain,
};
use super::common::get_next_byte_from_chain;
use super::DeserializationError;

#[derive(Clone, Debug, PartialEq)]
pub struct Int {
    pub number: BigInt,
    pub size: usize,
}

impl Int {
    pub fn new(number: i128, size: usize) -> Self {
        Self { number: BigInt::from(number), size }
    }
}

impl ABISerialized for Int {
    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        let vec = self.number.to_signed_bytes_be();
        let vec_bits_length = vec.len() * 8;

        let bitstring = if self.size > vec_bits_length {
            let padding = if self.number.sign() == Sign::Minus {
                0xFFu8
            } else {
                0u8
            };

            let dif = self.size - vec_bits_length;

            let mut vec_padding = Vec::new();
            vec_padding.resize(dif / 8 + 1, padding);

            let mut bitstring = Bitstring::create(vec_padding, dif);
            bitstring.append(&Bitstring::create(vec, self.size - dif));
            bitstring
        } else {
            let offset = vec_bits_length - self.size;
            Bitstring::create(vec, vec_bits_length).substring(offset..)
        };

        prepend_data_to_chain(destination, bitstring)
    }

    fn get_in_cell_size(&self) -> usize {
        self.size
    }
}

#[macro_export]
macro_rules! define_int_ABIParameter {
    ( $type:ident, $str_type:expr, $size: tt) => {

        impl ABISerialized for $type {

            fn prepend_to(&self, destination: BuilderData) -> BuilderData {
                let vec = self.to_be_bytes().to_vec();
                let size = vec.len();
                let data = Bitstring::create(vec, size * 8);

                prepend_data_to_chain(destination, data)
            }

            fn get_in_cell_size(&self) -> usize {
                $size * 8
            }
        }

        impl ABIDeserialized for $type {
            type Out = $type;

            fn read_from(
                cursor: SliceData,
            ) -> Result<(Self::Out, SliceData), DeserializationError> {
                let mut cursor = cursor;
                let mut bytes: [u8; $size] = [0x00; $size];
                for i in 0..$size {
                    let (byte, new_cursor) = get_next_byte_from_chain(cursor)?;
                    cursor = new_cursor;
                    bytes[i] = byte;
                }
                let decoded = Self::from_be_bytes(bytes);
                Ok((decoded, cursor))
            }
        }

        impl ABITypeSignature for $type {
            fn type_signature() -> String {
                $str_type.to_string()
            }
        }
    };
}

define_int_ABIParameter!(u8, "uint8", 1);
define_int_ABIParameter!(u16, "uint16", 2);
define_int_ABIParameter!(u32, "uint32", 4);
define_int_ABIParameter!(u64, "uint64", 8);
define_int_ABIParameter!(u128, "uint128", 16);
define_int_ABIParameter!(i8, "int8", 1);
define_int_ABIParameter!(i16, "int16", 2);
define_int_ABIParameter!(i32, "int32", 4);
define_int_ABIParameter!(i64, "int64", 8);
define_int_ABIParameter!(i128, "int128", 16);
