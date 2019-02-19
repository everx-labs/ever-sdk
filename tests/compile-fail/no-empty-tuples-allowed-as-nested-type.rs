// error-pattern: the trait `abi_lib::types::ABIParameter` is not implemented for `()`

extern crate abi_lib;

use abi_lib::abi_call::ABICall;

fn main() {
    let x = ABICall::<((),()), ()>::encode_function_call("foo", ((),()));
    println!("{:?}", x);   
}
