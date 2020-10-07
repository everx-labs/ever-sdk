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

use crate::client::*;
use std::ptr::null;

// Rust exported functions

pub fn create_context(config: String) -> JsonResponse {
    Client::shared().create_context(config)
}

pub fn destroy_context(context: InteropContext) {
    Client::shared().destroy_context(context)
}

pub fn json_sync_request(
    context: InteropContext,
    function: String,
    params_json: String,
) -> JsonResponse {
    Client::json_sync_request(
        context,
        function,
        params_json)
}

pub fn json_async_request(
    context: InteropContext,
    function: String,
    params_json: String,
    request_id: u32,
    on_result: OnResult,
) {
    Client::json_async_request(
        context,
        function,
        params_json,
        request_id,
        Box::new(
            move |request_id: u32, params_json: &str, response_type: u32, finished: bool| {
                on_result(request_id, params_json.into(), response_type, finished)
        }))
}

// C-library exported functions

pub type OnResult = extern fn(request_id: u32, params_json: InteropString, response_type: u32, finished: bool);

#[no_mangle]
pub unsafe extern "C" fn tc_create_context(config: InteropString) -> *const JsonResponse {
    let response = create_context(config.to_string());
    Box::into_raw(Box::new(response))
}

#[no_mangle]
pub unsafe extern "C" fn tc_destroy_context(context: InteropContext) {
    destroy_context(context)
}

#[no_mangle]
pub unsafe extern "C" fn tc_json_request(
    context: InteropContext,
    function: InteropString,
    params_json: InteropString,
    request_id: u32,
    on_result: OnResult,
) {
    json_async_request(
        context,
        function.to_string(),
        params_json.to_string(),
        request_id,
        on_result
    );
}

#[no_mangle]
pub unsafe extern "C" fn tc_json_request_sync(
    context: InteropContext,
    function: InteropString,
    params_json: InteropString,
) -> *const JsonResponse {
    let response = json_sync_request(
        context,
        function.to_string(),
        params_json.to_string());
    Box::into_raw(Box::new(response))
}

#[no_mangle]
pub unsafe extern "C" fn tc_destroy_json_response(
    response: *const JsonResponse
) {
    if response.is_null() {
        return;
    }
    let response = Box::from_raw(response as *mut JsonResponse);
    drop(response);
}

#[no_mangle]
pub unsafe extern "C" fn tc_read_json_response(
    response: *const JsonResponse
) -> InteropJsonResponse {
    if response.is_null() {
        InteropJsonResponse::default()
    } else {
        InteropJsonResponse::from(&*response)
    }
}

// Types

pub type InteropContext = u32;

#[repr(C)]
#[derive(Clone)]
pub struct InteropString {
    pub content: *const u8,
    pub len: u32,
}


#[repr(C)]
pub struct InteropJsonResponse {
    pub result_json: InteropString,
    pub error_json: InteropString,
}

#[derive(Serialize, Clone, Debug)]
pub struct JsonResponse {
    pub result_json: String,
    pub error_json: String,
}

impl JsonResponse {
    pub fn send(&self, on_result: &ExternalCallback, request_id: u32) {
        if !self.result_json.is_empty() {
            on_result(request_id, self.result_json.as_str(), ResponseType::Success as u32, true)
        } else {
            on_result(request_id, self.error_json.as_str(), ResponseType::Error as u32, true)
        }
    }
}

// Helpers

impl InteropString {
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

impl From<&String> for InteropString {
    fn from(s: &String) -> Self {
        Self {
            content: s.as_ptr(),
            len: s.len() as u32,
        }
    }
}

impl From<&str> for InteropString {
    fn from(s: &str) -> Self {
        Self {
            content: s.as_ptr(),
            len: s.len() as u32,
        }
    }
}

impl InteropJsonResponse {
    pub(crate) fn default() -> Self {
        Self {
            result_json: InteropString::default(),
            error_json: InteropString::default(),
        }
    }

    pub(crate) fn from(response: &JsonResponse) -> Self {
        Self {
            result_json: InteropString::from(&response.result_json),
            error_json: InteropString::from(&response.error_json),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn to_response(&self) -> JsonResponse {
        JsonResponse {
            result_json: self.result_json.to_string(),
            error_json: self.error_json.to_string(),
        }
    }
}

