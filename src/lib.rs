extern crate tonlabs_sdk_emulator;
extern crate crypto;

pub mod abi_parameter;
pub mod abi_call;

#[cfg(test)]
mod tests {
    use super::abi_call::ABICall;
    
    #[test]
    fn it_works() {
        let message = ABICall::<(i128,), (bool,)>::encode_function_call("IsEvenNumber".to_string(), 
            (-1123,));

        dbg!(message);
    }
}