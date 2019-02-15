use std::ops::Deref;
use std::borrow::Borrow;

use super::{
    ABIParameter, 
    DeserializationError
};

use tonlabs_sdk_emulator::bitstring::Bit;
use tonlabs_sdk_emulator::stack::{BuilderData, SliceData};

#[macro_export]
macro_rules! bits {
    ( $size:expr, $type:ident ) => {
        pub struct $type {
            data: [Bit;$size],
        }

        impl Deref for $type {
            type Target = [Bit; $size];

            fn deref(&self) -> &[Bit; $size] {
                &self.data
            }
        }

        impl Borrow<[Bit]> for $type {
            fn borrow(&self) -> &[Bit] {
                &self.data
            }
        }

        impl Borrow<[Bit; $size]> for $type {
            fn borrow(&self) -> &[Bit; $size] {
                &self.data
            }
        }

        impl ABIParameter for $type
        {
            type Out = Vec<<Bit as ABIParameter>::Out>;

            fn prepend_to(&self, destination: BuilderData) -> BuilderData {
                self.data.prepend_to(destination)
            }

            fn type_signature() -> String {
                format!("bits{}", $size)
            }

            fn get_in_cell_size(&self) -> usize {
                self.data.get_in_cell_size()
            }

            fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError> {
                let (bits, cursor) = <[Bit;$size] as ABIParameter>::read_from(cursor)?;
                
                Ok((bits, cursor))
            }
        }
    };
}

bits!(1, Bits1);
