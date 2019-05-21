use tvm::bitstring::Bitstring;
use tvm::stack::{BuilderData, SliceData, IBitstring};
use super::DeserializationError;
use types::ABIParameter;

#[macro_export]
macro_rules! makeOutParameter {
    ($t:tt, $($T: tt),+ ) => {
        impl<$($T),*> ABIOutParameter for $t<$($T),*>
        where
            $(
            $T: ABIParameter
            ),*
        {
            type Out = <Self as ABIParameter>::Out;

            fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError>
            {
                <Self as ABIParameter>::read_from(cursor)
            }
        }
    };
    ($t: ty) => {
        impl ABIOutParameter for $t {
            type Out = <Self as ABIParameter>::Out;

            fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError>
            {
                <Self as ABIParameter>::read_from(cursor)
            }
        }
    }
}

// put data to cell and make chain if data doesn't fit into cell
pub fn prepend_data_to_chain(mut builder: BuilderData, data: Bitstring) -> BuilderData {
    let mut data = data;

    while data.length_in_bits() > 0 {
        let remaining_bits = BuilderData::bits_capacity() - builder.bits_used();

        if remaining_bits > 0 {
            // data does not fit into cell - fill current cell and take remaining data
            if remaining_bits < data.length_in_bits() {
                let cut = data.substring(data.length_in_bits() - remaining_bits..data.length_in_bits());
                let mut vec = vec![];
                cut.into_bitstring_with_completion_tag(&mut vec);
                builder.prepend_bitstring(&vec).unwrap();

                data = data.substring(0..data.length_in_bits() - remaining_bits);
            } else {
                // data fit into current cell - no data remaining
                let mut vec = vec![];
                data.into_bitstring_with_completion_tag(&mut vec);
                builder.prepend_bitstring(&vec).unwrap();

                data.clear();
            }
        } else {
            // current cell is full - move to next
            let mut next_builder = BuilderData::new();
            next_builder.append_reference(builder);
            builder = next_builder;
        }
    }

    builder
}

pub fn get_next_byte_from_chain(cursor: SliceData) -> Result<(u8, SliceData), DeserializationError> {
    let mut cursor = cursor;
    
    if cursor.remaining_bits() >= 8 {
        Ok((cursor.get_next_byte(), cursor))
    }
    else {
        let mut result: u8 = 0;
        for i in (0..8).rev() {
            let (bit, new_cursor) = <bool as ABIParameter>::read_from(cursor)?;
            cursor = new_cursor;

            if bit {
                result |= 1 << i;
            }
        }

        Ok((result, cursor))
    }
}
