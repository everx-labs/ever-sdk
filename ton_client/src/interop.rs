use client::*;
use std::ptr::null;

// Functions

#[no_mangle]
pub unsafe extern "C" fn tc_create_context() -> InteropContext {
    Client::shared().create_context()
}

#[no_mangle]
pub unsafe extern "C" fn tc_destroy_context(context: InteropContext) {
    Client::shared().destroy_context(context)
}

#[no_mangle]
pub unsafe extern "C" fn tc_json_request(
    context: InteropContext,
    method_name: InteropString,
    params_json: InteropString,
) -> *const JsonResponse {
    let response = Client::shared().json_request(
        context,
        method_name.to_string(),
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
pub struct InteropString {
    pub content: *const u8,
    pub len: u32,
}


#[repr(C)]
pub struct InteropJsonResponse {
    pub result_json: InteropString,
    pub error_json: InteropString,
}

pub struct JsonResponse {
    pub result_json: String,
    pub error_json: String,
}

// Helpers

impl InteropString {
    pub(crate) fn default() -> Self {
        Self {
            content: null(),
            len: 0
        }
    }

    pub(crate) fn from(s: &String) -> Self {
        Self {
            content: s.as_ptr(),
            len: s.len() as u32,
        }
    }

    pub(crate) fn to_string(&self) -> String {
        unsafe {
            let utf8 = std::slice::from_raw_parts(self.content, self.len as usize);
            String::from_utf8(utf8.to_vec()).unwrap()
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

    pub(crate) fn to_response(&self) -> JsonResponse {
        JsonResponse {
            result_json: self.result_json.to_string(),
            error_json: self.error_json.to_string(),
        }
    }
}

