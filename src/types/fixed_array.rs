use super::common::*;
use super::common_arrays::*;
use super::{
    ABIParameter
};

use tvm::bitstring::{Bit, Bitstring};
use tvm::stack::{BuilderData};

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
macro_rules! fixed_abi_array {
    ( $size:expr, $type:ident ) => {
        
        #[derive(Clone)]
        pub struct $type<T> {
            pub data: [T;$size],
        }

        impl<T> PartialEq for $type<T>
        where
            T: PartialEq,
        {
            fn eq(&self, other: &$type<T>) -> bool {
                if self.len() != other.len() {
                    return false;
                }

                for i in 0..self.len() {
                    if self[i] != other[i] {
                        return false;
                    }
                }

                true
            }
        }

        impl<T> std::fmt::Debug for $type<T>
        where
            T: std::fmt::Debug,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.data.fmt(f)
            }
        }

        impl<T> From<[T;$size]> for $type<T> {
            fn from(array: [T;$size]) -> Self {
                $type{ data: array }
            }
        }

        impl<T> Into<[T;$size]> for $type<T> {
            fn into(self) -> [T;$size] {
                self.data
            }
        }

        impl<T> std::ops::Deref for $type<T> {
            type Target = [T; $size];

            fn deref(&self) -> &[T; $size] {
                &self.data
            }
        }

        impl<T> std::borrow::Borrow<[T]> for $type<T> {
            fn borrow(&self) -> &[T] {
                &self.data
            }
        }

        impl<T> std::borrow::Borrow<[T; $size]> for $type<T> {
            fn borrow(&self) -> &[T; $size] {
                &self.data
            }
        }

        impl<T> $crate::types::ABIParameter for $type<T>
        where
            T: $crate::types::ABIParameter,
        {
            type Out = Vec<T::Out>;

            fn prepend_to(&self, destination: tvm::stack::BuilderData) -> tvm::stack::BuilderData {
                $crate::types::prepend_fixed_array(destination, &self.data)
            }

            fn get_in_cell_size(&self) -> usize {
                let mut result = 0;
                for i in 0..$size {
                    result += self[i].get_in_cell_size();
                }

                // if array doesn't fit into cell it is put in separate chain and only 2 bits are put in main chain cell
                if result > tvm::stack::BuilderData::new().bits_capacity() {
                    2
                } else {
                    result + 2
                }
            }

            fn read_from(
                cursor: tvm::stack::SliceData,
            ) -> Result<(Self::Out, tvm::stack::SliceData), $crate::types::DeserializationError> {
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

        impl<T: $crate::types::ABITypeSignature> $crate::types::ABITypeSignature for $type<T> {
            fn type_signature() -> String {
                format!("{}[{}]", T::type_signature(), $size)
            }
        }
    };
}

fixed_abi_array!(1, AbiArray1);
fixed_abi_array!(2, AbiArray2);
fixed_abi_array!(3, AbiArray3);
fixed_abi_array!(4, AbiArray4);
fixed_abi_array!(5, AbiArray5);
fixed_abi_array!(6, AbiArray6);
fixed_abi_array!(7, AbiArray7);
fixed_abi_array!(8, AbiArray8);
fixed_abi_array!(9, AbiArray9);
fixed_abi_array!(10, AbiArray10);
fixed_abi_array!(11, AbiArray11);
fixed_abi_array!(12, AbiArray12);
fixed_abi_array!(13, AbiArray13);
fixed_abi_array!(14, AbiArray14);
fixed_abi_array!(15, AbiArray15);
fixed_abi_array!(16, AbiArray16);
fixed_abi_array!(17, AbiArray17);
fixed_abi_array!(18, AbiArray18);
fixed_abi_array!(19, AbiArray19);
fixed_abi_array!(20, AbiArray20);
fixed_abi_array!(21, AbiArray21);
fixed_abi_array!(22, AbiArray22);
fixed_abi_array!(23, AbiArray23);
fixed_abi_array!(24, AbiArray24);
fixed_abi_array!(25, AbiArray25);
fixed_abi_array!(26, AbiArray26);
fixed_abi_array!(27, AbiArray27);
fixed_abi_array!(28, AbiArray28);
fixed_abi_array!(29, AbiArray29);
fixed_abi_array!(30, AbiArray30);
fixed_abi_array!(31, AbiArray31);
fixed_abi_array!(32, AbiArray32);
