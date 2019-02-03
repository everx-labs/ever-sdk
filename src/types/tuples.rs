#![allow(non_snake_case)]

use super::{ABIParameter, DeserializationError};

use tonlabs_sdk_emulator::stack::{BuilderData, SliceData};

impl ABIParameter for () {
    type Out = ();

    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        destination
    }

    fn type_signature() -> String {
        String::from("()")
    }

    fn get_in_cell_size(&self) -> usize {
        0
    }

    fn read_from(cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
        Ok(((), cursor))
    }

    fn is_restricted_to_root() -> bool {
        true
    }
}

macro_rules! tuple {
    (@expand_prepend_to $destination:ident, $x:ident) => {{
        $x.prepend_to($destination)
    }};
    (@expand_prepend_to $destination:ident, $x:ident, $($other:ident),+) => {{
        let next = tuple!(@expand_prepend_to $destination, $($other),*);
        $x.prepend_to(next)
    }};

    (@expand_get_in_cell_size $x:ident) => {{
        $x.get_in_cell_size()
    }};
    (@expand_get_in_cell_size $x:ident, $($other:ident),+) => {{
        $x.get_in_cell_size() + tuple!(@expand_get_in_cell_size $($other),*)
    }};

    ($($T:tt),*) => {
        impl<$($T),*> ABIParameter for ($($T,)*)
        where
            $($T: ABIParameter),*
        {
            type Out = ($($T::Out,)*);

            fn prepend_to(&self, destination: BuilderData) -> BuilderData {
                let ($($T,)*) = self;
                let destination = tuple!(@expand_prepend_to destination,  $($T),*);
                destination
            }

            fn type_signature() -> String {
                let mut result = "".to_owned()
                $(
                    + "," + &$T::type_signature()
                )*
                    + ")";
                result.replace_range(..1, "(");
                result
            }

            fn get_in_cell_size(&self) -> usize {
                let ($($T,)*) = self;
                tuple!(@expand_get_in_cell_size $($T),*)
            }

            fn read_from(cursor: SliceData) -> Result<(Self::Out, SliceData), DeserializationError> {
                let mut reader = $crate::types::reader::Reader::new(cursor);
                Ok((
                    ($(
                        reader.read_next::<$T>()?,
                    )*),
                    reader.remainder()
                ))
            }
        }
    };

}

tuple!(T1);
tuple!(T1, T2);
tuple!(T1, T2, T3);
tuple!(T1, T2, T3, T4);
tuple!(T1, T2, T3, T4, T5);
