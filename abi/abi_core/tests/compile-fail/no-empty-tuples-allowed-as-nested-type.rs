// error-pattern: the trait bound `(): ton_abi_core::types::ABISerialized` is not satisfied

extern crate ton_abi_core;

use ton_abi_core::abi_call::ABICall;

fn main() {
    let x = ABICall::<((),()), ()>::encode_function_call("foo", ((),()));
    println!("{:?}", x);   
}
