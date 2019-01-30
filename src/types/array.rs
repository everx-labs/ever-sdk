use super::common::prepend_array;
use super::ABIParameter;
use tonlabs_sdk_emulator::stack::BuilderData;

#[macro_export]
macro_rules! define_array_ABIParameter {
    ( $size:expr ) => {
        impl<T> ABIParameter for [T; $size]
        where
            T: ABIParameter,
        {
            fn prepend_to(&self, destination: BuilderData) -> BuilderData {
                prepend_array(destination, self, false)
            }

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
