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

use self::ton_client::{
    create_context,
    destroy_context,
    json_sync_request,
    InteropContext,
    InteropString,
    tc_json_request_async,
    OnResult
};

#[no_mangle]
pub unsafe extern fn core_create_context() -> u32 {
    create_context()
}

#[no_mangle]
pub unsafe extern fn core_destroy_context(context: u32) {
    destroy_context(context)
}

#[no_mangle]
pub unsafe extern fn core_request(
    context: u32,
    method: &InteropString,
    params_json: &InteropString,
    request_id: i32,
    on_result: OnResult,
) {
    tc_json_request_async(context, method, params_json, request_id, on_result)
}

#[no_mangle]
pub unsafe extern fn ton_sdk_json_rpc_request(
    method: &InteropString,
    params_json: &InteropString,
    request_id: i32,
    on_result: OnResult,
) {
    let context = create_context();
    tc_json_request_async(context, method, params_json, request_id, on_result);
    destroy_context(context)
}

