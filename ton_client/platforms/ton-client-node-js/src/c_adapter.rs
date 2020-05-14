/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

extern crate ton_client;
use self::ton_client::*;

#[no_mangle]
pub unsafe extern fn ton_sdk_utf8_string(s: *mut String) -> *mut TonSdkUtf8String {
    Box::into_raw(Box::new(TonSdkUtf8String::from(&**s)))
}

#[no_mangle]
pub unsafe extern fn ton_sdk_rust_string_destroy(s: *mut String) {
    let _ = Box::from_raw(s);
}

#[no_mangle]
pub unsafe extern fn ton_sdk_utf8_string_destroy(s: *mut TonSdkUtf8String) {
    let _ = Box::from_raw(s);
}

use libc::size_t;

#[repr(C)]
pub struct TonSdkUtf8String {
    pub ptr: *const u8,
    pub len: size_t,
}

impl<'a> From<&'a str> for TonSdkUtf8String {
    fn from(s: &'a str) -> Self {
        TonSdkUtf8String {
            ptr: s.as_ptr(),
            len: s.len() as size_t,
        }
    }
}

impl TonSdkUtf8String {
    pub fn as_str(&self) -> &str {
        use std::{slice, str};

        unsafe {
            let slice = slice::from_raw_parts(self.ptr, self.len);
            str::from_utf8(slice).unwrap()
        }
    }
}

enum OnResultFlags {
    Finished = 1,
}

type OnResult = extern fn(request_id: i32, result_json: TonSdkUtf8String, error_json: TonSdkUtf8String, flags: i32);
/*
struct CResultHandler {
    request_id: i32,
    on_result: OnResult,
}

impl CResultHandler {
    fn new(request_id: i32, on_result: OnResult) -> CResultHandler {
        CResultHandler { request_id, on_result }
    }
}

impl ResultHandler for CResultHandler {
    fn on_result(&self, result_json: String, error_json: String, flags: i32) {
        let result = TonSdkUtf8String::from(result_json.as_str());
        let error = TonSdkUtf8String::from(error_json.as_str());
        let on_result = self.on_result;
        on_result(self.request_id, result, error, flags)
    }
}*/

#[no_mangle]
pub unsafe extern fn ton_sdk_json_rpc_request(
    //context: InteropContext,
    method: *mut TonSdkUtf8String,
    params_json: *mut TonSdkUtf8String,
    request_id: i32,
    on_result: OnResult,
) {
    let context = create_context();
    let response = json_sync_request(
        context,
        String::from((*method).as_str()),
        String::from((*params_json).as_str()),
    );

    let result = TonSdkUtf8String::from(response.result_json.as_str());
    let error = TonSdkUtf8String::from(response.error_json.as_str());
    on_result(request_id, result, error, OnResultFlags::Finished as i32);
    destroy_context(context);
}
