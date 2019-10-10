use num_bigint::{BigInt, BigUint};
use tvm::stack::{BuilderData, SliceData};
use types::{
    ABIDeserialized,
    ABISerialized,
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

#[derive(Clone, Debug, PartialEq)]
pub struct Uint {
    pub number: BigUint,
    pub size: usize,
}


impl Int {
    pub fn new(number: i128, size: usize) -> Self {
        Self { number: BigInt::from(number), size }
    }
}


impl Uint {
    pub fn new(number: u128, size: usize) -> Self {
        Self { number: BigUint::from(number), size }
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
