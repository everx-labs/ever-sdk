#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate lazy_static;
extern crate ton_sdk;

mod error;
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
