#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

extern crate rand;
extern crate ed25519_dalek;
extern crate num_bigint;
extern crate sha2;
extern crate bip39;
extern crate hmac;
extern crate pbkdf2;
extern crate base58;
extern crate byteorder;
extern crate secp256k1;
extern crate ton_sdk;
extern crate tvm;

mod types;
mod dispatch;
mod client;
mod setup;
mod contracts;
mod crypto;
mod queries;

mod interop;
pub use self::interop::*;


#[test]
fn test() {
    unsafe {
        let context = tc_create_context();

        let version = tc_json_request(context,
            InteropString::from(&"version".to_string()),
            InteropString::from(&"".to_string()));
        let v = tc_read_json_response(version);
        println!("result: {}", v.result_json.to_string());
        println!("error: {}", v.error_json.to_string());
        tc_destroy_json_response(version);

        tc_destroy_context(context);
    }
}
