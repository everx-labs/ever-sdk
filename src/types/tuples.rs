#![allow(non_snake_case)]

use super::ABIParameter;

use tonlabs_sdk_emulator::stack::BuilderData;

impl ABIParameter for () {
    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        destination
    }

    fn type_signature() -> String {
        String::from("()")
    }

    fn get_in_cell_size(&self) -> usize {
        0
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
        }
    };

}

tuple!(T1);
tuple!(T1, T2);
tuple!(T1, T2, T3);
tuple!(T1, T2, T3, T4);
tuple!(T1, T2, T3, T4, T5);
