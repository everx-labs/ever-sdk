use tonlabs_sdk_emulator::stack::BuilderData;
use tonlabs_sdk_emulator::bitstring::{Bit, Bitstring};

pub trait ABIParameter {
    fn write_to<T: ABIParameter>(&self, builder: BuilderData, remain_params: Option<T>) -> BuilderData;
    fn type_signature() -> String;
}


// appends data to cell and creates new cell, if data overflows
pub fn append_data<T: ABIParameter>(mut builder: BuilderData, data: Bitstring, next_param: Option<T>, remain_params: Option<T>) -> BuilderData {

    let remaining_bits = builder.bits_capacity() - builder.bits_used();
    let mut rem: Bitstring = Bitstring::new();

    if remaining_bits > 0 {
        // data does not fit into cell - fill current cell and take remaining data to `rem`
        if remaining_bits < data.length_in_bits(){
            let mut cut = Bitstring::new();
            data.bits(0 .. remaining_bits).data.iter().for_each(|x| { cut.append_bit(x); });
            builder.append_data(&cut);

            data.bits(remaining_bits .. data.length_in_bits()).data.iter().for_each(|x| { rem.append_bit(x); });
        }
        else{
            // data fit into current cell - no data remaining
            builder.append_data(&data);
        }
    }
    else{
        // current cell full - all data remaining
        rem = data;
    }

    if rem.length_in_bits() > 0 {
        // there is remainig data - create new cell and continue with it
        let mut next_builder = BuilderData::new();

        next_builder = append_data(next_builder, rem, next_param, remain_params);

        builder.append_reference(next_builder);
    }
    else {
        // data ended - continue with new parameter or finish recursion if no more parameters
        builder = match next_param {
            Some(param) => param.write_to(builder, remain_params),
            None => builder
        }
    }

    builder
}

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

impl ABIParameter for bool {

    fn write_to<T: ABIParameter>(&self, builder: BuilderData, remain_params: Option<T>) -> BuilderData {
        let mut bitstring = Bitstring::new();
        bitstring.append_bit(if *self {&Bit::One} else {&Bit::Zero});
        
        append_data(builder, bitstring, remain_params, None)
    }

    fn type_signature() -> String {
        "bool".to_string()
    }
}

impl ABIParameter for () {

    fn write_to<T: ABIParameter>(&self, builder: BuilderData, remain_params: Option<T>) -> BuilderData {
        let bitstring = Bitstring::new();
        
        append_data(builder, bitstring, remain_params, None)
    }

    fn type_signature() -> String {
        "".to_string()
    }
}

impl<A> ABIParameter for &A where A: ABIParameter {

    fn write_to<T: ABIParameter>(&self, builder: BuilderData, remain_params: Option<T>) -> BuilderData {
        (**self).write_to(builder, remain_params)
    }

    fn type_signature() -> String {
       A::type_signature()
    }
}

impl<A> ABIParameter for (A,) where A: ABIParameter {
    
    fn write_to<T: ABIParameter>(&self, builder: BuilderData, remain_params: Option<T>) -> BuilderData {
        self.0.write_to(builder, remain_params)
    }

    fn type_signature() -> String {
        "(".to_string() + &A::type_signature() + ")"
    }
}

impl<A, B> ABIParameter for (A, B) where A: ABIParameter, B: ABIParameter {
    
    fn write_to<T: ABIParameter>(&self, builder: BuilderData, remain_params: Option<T>) -> BuilderData {
        let (a, b) = self;
        
        match remain_params {
            Some(params) => a.write_to(builder, Some((b, params))),
            None => a.write_to(builder, Some(b))
        }
    }

    fn type_signature() -> String {
        "(".to_string() + &A::type_signature() + "," + &B::type_signature() + ")"
    }
}

impl<A, B, C> ABIParameter for (A, B, C) where A: ABIParameter, B: ABIParameter, C: ABIParameter {

    fn write_to<T: ABIParameter>(&self, builder: BuilderData, remain_params: Option<T>) -> BuilderData {
        let (a, b, c) = self;
        match remain_params {
            Some(params) => a.write_to(builder, Some((b, c, params))),
            None => a.write_to(builder, Some((b, c)))
        }
    }

    fn type_signature() -> String {
        "(".to_string() + &A::type_signature() + "," + &B::type_signature() + "," + &C::type_signature() + ")"
    }
}
