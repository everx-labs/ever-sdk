use super::common::*;
use super::common_arrays::*;
use super::ABISerialized;

use tvm::bitstring::{Bit, Bitstring};
use tvm::stack::{BuilderData};

// put fixed array to chain or to separate branch depending on array size
pub fn prepend_fixed_array<T: ABISerialized>(
    mut destination: BuilderData,
    array: &[T],
) -> BuilderData {
    let mut array_size = 0;
    for i in array {
        array_size += i.get_in_cell_size();
    }

    // if array doesn't fit into one cell, we put into separate chain
    if array_size > BuilderData::bits_capacity() {
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

pub fn get_fixed_array_in_cell_size<T: ABISerialized>(array: &[T]) -> usize {
    let mut result = 0;
    for item in array {
        result += item.get_in_cell_size();
    }

    // if array doesn't fit into cell it is put in separate chain and only 2 bits are put in main chain cell
    if result > tvm::stack::BuilderData::bits_capacity() {
        2
    } else {
        result + 2
    }
}

#[macro_export]
macro_rules! fixed_abi_array {
    ( $inner_type:ty, $size:expr, $type:ident ) => {
        
        #[derive(Clone)]
        pub struct $type {
            pub data: [$inner_type;$size],
        }

        impl PartialEq for $type
        {
            fn eq(&self, other: &$type) -> bool {
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

        impl std::fmt::Debug for $type
        {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.data.fmt(f)
            }
        }

        impl From<[$inner_type;$size]> for $type {
            fn from(array: [$inner_type;$size]) -> Self {
                $type{ data: array }
            }
        }

        impl Into<[$inner_type;$size]> for $type {
            fn into(self) -> [$inner_type;$size] {
                self.data
            }
        }

        impl std::ops::Deref for $type {
            type Target = [$inner_type; $size];

            fn deref(&self) -> &[$inner_type; $size] {
                &self.data
            }
        }

        impl std::borrow::Borrow<[$inner_type]> for $type {
            fn borrow(&self) -> &[$inner_type] {
                &self.data
            }
        }

        impl std::borrow::Borrow<[$inner_type; $size]> for $type {
            fn borrow(&self) -> &[$inner_type; $size] {
                &self.data
            }
        }

        impl $crate::types::ABISerialized for $type
        {
            fn prepend_to(&self, destination: tvm::stack::BuilderData) -> tvm::stack::BuilderData {
                $crate::types::prepend_fixed_array(destination, &self.data)
            }

            fn get_in_cell_size(&self) -> usize {
                $crate::types::get_fixed_array_in_cell_size(&self.data)
            }
        }

        impl $crate::types::ABIDeserialized for $type
        {
            type Out = Vec<<$inner_type as $crate::types::ABIDeserialized>::Out>;

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
                        let array = cursor.checked_drain_reference().unwrap();
                        let mut array = $crate::types::reader::Reader::new(array);
                        let mut result = vec![];
                        for _ in 0..$size {
                            result.push(array.read_next::<$inner_type>()?);
                        }
                        if !array.is_empty() {
                            return Err($crate::types::DeserializationError::with(array.remainder()));
                        }
                        Ok((result, cursor))
                    }
                    (true, false) => {
                        let mut result = vec![];
                        for _ in 0..$size {
                            result.push(cursor.read_next::<$inner_type>()?);
                        }
                        Ok((result, cursor.remainder()))
                    }
                    _ => Err($crate::types::DeserializationError::with(cursor.remainder())),
                }
            }
        }

        impl $crate::types::ABITypeSignature for $type {
            fn type_signature() -> String {
                <$inner_type as $crate::types::ABITypeSignature>::type_fixed_array_signature($size)
            }
        }
    };
}
