use super::common::*;
use super::common_arrays::*;
use super::{
    ABIParameter
};

use tvm::bitstring::{Bit, Bitstring};
use tvm::stack::{BuilderData, SliceData};

// put fixed array to chain or to separate branch depending on array size
pub fn prepend_fixed_array<T: ABIParameter>(
    mut destination: BuilderData,
    array: &[T],
) -> BuilderData {
    let mut array_size = 0;
    for i in array {
        array_size += i.get_in_cell_size();
    }

    // if array doesn't fit into one cell, we put into separate chain
    if array_size > destination.bits_capacity() {
        destination = put_array_to_separate_branch(destination, array);
    } else {
        // if array fit into cell data, put in into main chain
        destination = prepend_array_items_to_chain(destination, array);

        let mut bitstring = Bitstring::new();
        bitstring.append_bit(&Bit::One);
        bitstring.append_bit(&Bit::Zero);

        destination = prepend_data_to_chain(destination, bitstring);
    }

    destination
}

#[macro_export]
macro_rules! define_array_ABIParameter {
    ( $size:expr ) => {
        impl<T> $crate::types::ABIParameter for [T; $size]
        where
            T: $crate::types::ABIParameter,
        {
            type Out = Vec<T::Out>;

            fn prepend_to(&self, destination: BuilderData) -> BuilderData {
                $crate::types::prepend_fixed_array(destination, self)
            }

            fn get_in_cell_size(&self) -> usize {
                let mut result = 0;
                for i in 0..$size {
                    result += self[i].get_in_cell_size();
                }

                // if array doesn't fit into cell it is put in separate chain and only 2 bits are put in main chain cell
                if result > BuilderData::new().bits_capacity() {
                    2
                } else {
                    result + 2
                }
            }

            fn read_from(
                cursor: SliceData,
            ) -> Result<(Self::Out, SliceData), $crate::types::DeserializationError> {
                let mut cursor = $crate::types::reader::Reader::new(cursor);
                let flag = cursor.read_next::<(bool, bool)>()?;
                match flag {
                    (false, false) => {
                        let mut cursor = cursor.remainder();
                        if cursor.remaining_references() == 0 {
                            return Err($crate::types::DeserializationError::with(cursor));
                        }
                        let mut array = cursor.drain_reference();
                        let mut array = $crate::types::reader::Reader::new(array);
                        let mut result = vec![];
                        for _ in 0..$size {
                            result.push(array.read_next::<T>()?);
                        }
                        if !array.is_empty() {
                            return Err($crate::types::DeserializationError::with(array.remainder()));
                        }
                        Ok((result, cursor))
                    }
                    (true, false) => {
                        let mut result = vec![];
                        for _ in 0..$size {
                            result.push(cursor.read_next::<T>()?);
                        }
                        Ok((result, cursor.remainder()))
                    }
                    _ => Err($crate::types::DeserializationError::with(cursor.remainder())),
                }
            }
        }

        impl<T: $crate::types::ABITypeSignature> $crate::types::ABITypeSignature for [T; $size] {
            fn type_signature() -> String {
                format!("{}[{}]", T::type_signature(), $size)
            }
        }
    };
}

define_array_ABIParameter!(1);
define_array_ABIParameter!(2);
define_array_ABIParameter!(3);
define_array_ABIParameter!(4);
define_array_ABIParameter!(5);
define_array_ABIParameter!(6);
define_array_ABIParameter!(7);
define_array_ABIParameter!(8);
define_array_ABIParameter!(9);
define_array_ABIParameter!(10);
define_array_ABIParameter!(11);
define_array_ABIParameter!(12);
define_array_ABIParameter!(13);
define_array_ABIParameter!(14);
define_array_ABIParameter!(15);
define_array_ABIParameter!(16);
define_array_ABIParameter!(17);
define_array_ABIParameter!(18);
define_array_ABIParameter!(19);
define_array_ABIParameter!(20);
define_array_ABIParameter!(21);
define_array_ABIParameter!(22);
define_array_ABIParameter!(23);
define_array_ABIParameter!(24);
define_array_ABIParameter!(25);
define_array_ABIParameter!(26);
define_array_ABIParameter!(27);
define_array_ABIParameter!(28);
define_array_ABIParameter!(29);
define_array_ABIParameter!(30);
define_array_ABIParameter!(31);
define_array_ABIParameter!(32);
