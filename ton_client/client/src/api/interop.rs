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

use crate::api::get_dispatcher;
use crate::client::{Client, ContextHandle, Error, Request, ResponseHandler, StringData};
use crate::error::ApiResult;
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone)]
pub struct ResultOfCreateContext {
    pub handle: ContextHandle,
}

// C-library exported functions

unsafe fn sync_response(result: ApiResult<Value>) -> *const String {
    let response = match result {
        Ok(result) => json!({ "result": result }).to_string(),
        Err(err) => json!({ "error": err }).to_string(),
    };
    Box::into_raw(Box::new(response))
}

#[no_mangle]
pub unsafe extern "C" fn tc_create_context(config: StringData) -> *const String {
    sync_response(
        Client::shared()
            .create_context(config.to_string())
            .map(|x| Value::from(x.handle)),
    )
}

#[no_mangle]
pub unsafe extern "C" fn tc_destroy_context(context: ContextHandle) {
    Client::shared().destroy_context(context)
}

#[no_mangle]
pub unsafe extern "C" fn tc_request(
    context: ContextHandle,
    function: StringData,
    params_json: StringData,
    request_id: u32,
    response_handler: ResponseHandler,
) {
    let context_handle = context;
    let context = Client::shared().required_context(context);
    match context {
        Ok(context) => get_dispatcher().async_dispatch(
            context,
            function.to_string(),
            params_json.to_string(),
            request_id,
            response_handler,
        ),
        Err(_) => Request::new(response_handler, request_id)
            .finish_with_error(Error::invalid_context_handle(context_handle)),
    }
}

#[no_mangle]
pub unsafe extern "C" fn tc_request_sync(
    context: ContextHandle,
    function: StringData,
    params_json: StringData,
) -> *const String {
    let context_handle = context;
    let context = Client::shared().required_context(context);
    let result_value = match context {
        Ok(context) => {
            match get_dispatcher().sync_dispatch(
                context,
                function.to_string(),
                params_json.to_string(),
            ) {
                Ok(result_json) => serde_json::from_str(&result_json)
                    .map_err(|err| Error::cannot_serialize_result(err)),
                Err(err) => Err(err),
            }
        }
        Err(_) => Err(Error::invalid_context_handle(context_handle)),
    };
    sync_response(result_value)
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
        StringData::from(&*string)
    }
}
