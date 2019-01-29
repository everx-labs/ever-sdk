use tonlabs_sdk_emulator::stack::{
    BuilderData, 
};
use tonlabs_sdk_emulator::bitstring::{Bit, Bitstring};
use super::ABIParameter;
use super::common::prepend_data;


impl ABIParameter for ()
{
    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        destination
    }

    fn type_signature() -> String {
        String::from("()")
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
    
    ($($x:ident : $T:tt),*) => {
        impl<$($T),*> ABIParameter for ($($T,)*) 
        where
            $($T: ABIParameter),*
        {
            fn prepend_to(&self, destination: BuilderData) -> BuilderData {
                let ($($x),*) = self;
                let destination = tuple!(@expand_prepend_to destination,  $($x),*);
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
        }
    }
}

tuple!(a: T1);
tuple!(a: T1, b: T2);
tuple!(a: T1, b: T2, c: T3);
tuple!(a: T1, b: T2, c: T3, d: T4);

