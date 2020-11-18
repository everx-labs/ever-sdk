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
 *
 */

use super::request::Request;
use super::runtime::Runtime;
use crate::client::Error;
use crate::error::ClientResult;
use serde_json::Value;
use std::ffi::c_void;
use std::ptr::null;

pub type ContextHandle = u32;

#[derive(Serialize, Deserialize, Clone, num_derive::FromPrimitive)]
pub enum ResponseType {
    Success = 0,
    Error = 1,
    Nop = 2,
    AppRequest = 3,
    AppNotify = 4,
    Custom = 100,
}

// Rust-style interface

pub fn create_context(config: String) -> String {
    let context = Runtime::create_context(&config.to_string());
    convert_result_to_sync_response(context.map(|x| Value::from(x)))
}

pub fn destroy_context(context: ContextHandle) {
    Runtime::destroy_context(context)
}

pub type ResponseHandler =
    fn(request_id: u32, params_json: String, response_type: u32, finished: bool);

pub fn request(
    context: ContextHandle,
    function_name: String,
    params_json: String,
    request_id: u32,
    response_handler: ResponseHandler,
) {
    dispatch_request(
        context,
        function_name,
        params_json,
        Request::new(request_id, response_handler),
    )
}

pub type ResponseHandlerPtr =
    fn(request_ptr: *const (), params_json: String, response_type: u32, finished: bool);

pub fn request_ptr(
    context: ContextHandle,
    function_name: String,
    params_json: String,
    request_ptr: *const (),
    response_handler: ResponseHandlerPtr,
) {
    dispatch_request(
        context,
        function_name,
        params_json,
        Request::new_with_ptr(request_ptr, response_handler),
    )
}

pub fn request_sync(context: ContextHandle, function_name: String, params_json: String) -> String {
    let context_handle = context;
    let context = Runtime::required_context(context);
    let result_value = match context {
        Ok(context) => match Runtime::dispatch_sync(context, function_name, params_json) {
            Ok(result_json) => serde_json::from_str(&result_json)
                .map_err(|err| Error::cannot_serialize_result(err)),
            Err(err) => Err(err),
        },
        Err(_) => Err(Error::invalid_context_handle(context_handle)),
    };
    convert_result_to_sync_response(result_value)
}

// C-style interface

#[no_mangle]
pub unsafe extern "C" fn tc_create_context(config: StringData) -> *const String {
    Box::into_raw(Box::new(create_context(config.to_string())))
}

#[no_mangle]
pub unsafe extern "C" fn tc_destroy_context(context: ContextHandle) {
    destroy_context(context)
}

pub type CResponseHandler =
    extern "C" fn(request_id: u32, params_json: StringData, response_type: u32, finished: bool);

#[no_mangle]
pub unsafe extern "C" fn tc_request(
    context: ContextHandle,
    function_name: StringData,
    params_json: StringData,
    request_id: u32,
    response_handler: CResponseHandler,
) {
    dispatch_request(
        context,
        function_name.to_string(),
        params_json.to_string(),
        Request::new_with_c_handler(request_id, response_handler),
    )
}

pub type CResponseHandlerPtr = extern "C" fn(
    request_ptr: *const c_void,
    params_json: StringData,
    response_type: u32,
    finished: bool,
);

#[no_mangle]
pub unsafe extern "C" fn tc_request_ptr(
    context: ContextHandle,
    function_name: StringData,
    params_json: StringData,
    request_ptr: *const c_void,
    response_handler: CResponseHandlerPtr,
) {
    dispatch_request(
        context,
        function_name.to_string(),
        params_json.to_string(),
        Request::new_with_c_handler_ptr(request_ptr, response_handler),
    )
}

#[no_mangle]
pub unsafe extern "C" fn tc_request_sync(
    context: ContextHandle,
    function_name: StringData,
    params_json: StringData,
) -> *const String {
    Box::into_raw(Box::new(request_sync(
        context,
        function_name.to_string(),
        params_json.to_string(),
    )))
}

#[no_mangle]
pub unsafe extern "C" fn tc_destroy_string(string: *const String) {
    if string.is_null() {
        return;
    }
    let string = Box::from_raw(string as *mut String);
    drop(string);
}

#[no_mangle]
pub unsafe extern "C" fn tc_read_string(string: *const String) -> StringData {
    if string.is_null() {
        StringData::default()
    } else {
        StringData::new(&*string)
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct StringData {
    pub content: *const u8,
    pub len: u32,
}

impl StringData {
    pub fn new(s: &String) -> Self {
        Self {
            content: s.as_ptr(),
            len: s.len() as u32,
        }
    }

    pub fn default() -> Self {
        Self {
            content: null(),
            len: 0,
        }
    }

    pub fn to_string(&self) -> String {
        unsafe {
            let utf8 = std::slice::from_raw_parts(self.content, self.len as usize);
            String::from_utf8(utf8.to_vec()).unwrap()
        }
    }
}

// Internals

fn convert_result_to_sync_response(result: ClientResult<Value>) -> String {
    match result {
        Ok(result) => json!({ "result": result }).to_string(),
        Err(err) => json!({ "error": err }).to_string(),
    }
}

fn dispatch_request(
    context: ContextHandle,
    function_name: String,
    params_json: String,
    request: Request,
) {
    let context_handle = context;
    let context = Runtime::required_context(context);
    match context {
        Ok(context) => Runtime::dispatch_async(
            context,
            function_name.to_string(),
            params_json.to_string(),
            request,
        ),
        Err(_) => request.finish_with_error(Error::invalid_context_handle(context_handle)),
    }
}
