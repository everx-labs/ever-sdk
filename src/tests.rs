use crate::abi_call::ABICall;

#[test]
fn it_works() {
    let message = ABICall::<(i128,bool), (bool,)>::encode_function_call("IsEvenNumber".to_string(), 
                            (-1123,true));

    dbg!(message);
}
