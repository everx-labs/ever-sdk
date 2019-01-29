use tonlabs_sdk_emulator::stack::BuilderData;
use tonlabs_sdk_emulator::bitstring::{Bit, Bitstring};


//TODO:


#[macro_export]
macro_rules! define_int_ABIParameter {
    ( $type:ident, $str_type:expr) => {
        impl ABIParameter for $type {
            fn write_to<T: ABIParameter>(&self, builder: BuilderData, remain_params: Option<T>) -> BuilderData {
                let vec = self.to_be_bytes().to_vec();
                let size = vec.len();
                let bitstring = Bitstring::create(vec, size * 8);
                
                append_data(builder, bitstring, remain_params, None)
            }

            fn type_signature() -> String {
                $str_type.to_string()
            }
        }
    }
}

define_int_ABIParameter!(u8, "uint8");
define_int_ABIParameter!(u16, "uint16");
define_int_ABIParameter!(u32, "uint32");
define_int_ABIParameter!(u64, "uint64");
define_int_ABIParameter!(u128, "uint128");
define_int_ABIParameter!(i8, "int8");
define_int_ABIParameter!(i16, "int16");
define_int_ABIParameter!(i32, "int32");
define_int_ABIParameter!(i64, "int64");
define_int_ABIParameter!(i128, "int128");

