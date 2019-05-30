// error-pattern: the trait `ton_abi_core::types::ABIParameter` is not implemented for `()`

extern crate abi_lib;

use ton_abi_core::abi_call::ABICall;

fn main() {
    let x = ABICall::<((),()), ()>::encode_function_call("foo", ((),()));
    println!("{:?}", x);   
}
