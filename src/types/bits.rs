use tvm::bitstring::Bit;
use tvm::stack::{BuilderData, SliceData};

#[macro_export]
macro_rules! bits {
    ( $size:expr, $type:ident ) => {

        #[derive(Clone)]
        pub struct $type {
            pub data: [Bit;$size],
        }

        impl From<[Bit;$size]> for $type {
            fn from(array: [Bit;$size]) -> Self {
                $type{ data: array}
            }
        }

        impl std::ops::Deref for $type {
            type Target = [Bit; $size];

            fn deref(&self) -> &[Bit; $size] {
                &self.data
            }
        }

        impl std::borrow::Borrow<[Bit]> for $type {
            fn borrow(&self) -> &[Bit] {
                &self.data
            }
        }

        impl std::borrow::Borrow<[Bit; $size]> for $type {
            fn borrow(&self) -> &[Bit; $size] {
                &self.data
            }
        }

        impl $crate::types::ABIParameter for $type
        {
            type Out = Vec<<Bit as $crate::types::ABIParameter>::Out>;

            fn prepend_to(&self, destination: BuilderData) -> BuilderData {
                $crate::types::prepend_fixed_array(destination, &self.data)
            }

            fn get_in_cell_size(&self) -> usize {
                // if array doesn't fit into cell it is put in separate chain and only 2 bits are put in main chain cell
                if self.len() > BuilderData::new().bits_capacity() {
                    2
                } else {
                    self.len() + 2
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
                            result.push(array.read_next::<Bit>()?);
                        }
                        if !array.is_empty() {
                            return Err($crate::types::DeserializationError::with(array.remainder()));
                        }
                        Ok((result, cursor))
                    }
                    (true, false) => {
                        let mut result = vec![];
                        for _ in 0..$size {
                            result.push(cursor.read_next::<Bit>()?);
                        }
                        Ok((result, cursor.remainder()))
                    }
                    _ => Err($crate::types::DeserializationError::with(cursor.remainder())),
                }
            }
        }

        impl $crate::types::ABITypeSignature for $type {
            fn type_signature() -> String {
                format!("bits{}", $size)
            }
        }
    };
}

bits!(8, Bits8);
bits!(16, Bits16);
bits!(32, Bits32);
bits!(64, Bits64);
bits!(128, Bits128);
bits!(256, Bits256);
bits!(512, Bits512);
bits!(1024, Bits1024);
