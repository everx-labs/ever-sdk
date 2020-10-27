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

pub use self::ton_client::{
    tc_create_context,
    tc_destroy_context,
    tc_json_request_async,
    InteropContext,
    StringData,
    CResponseHandler
};

// Obsolete. Used for backward compatibility only.
//
#[no_mangle]
pub unsafe extern fn ton_sdk_json_rpc_request(
    method: &StringData,
    params_json: &StringData,
    request_id: i32,
    on_result: CResponseHandler,
) {
    let context = self::ton_client::create_context();
    self::ton_client::tc_json_request_async(
        context,
        (*method).clone(),
        (*params_json).clone(),
        request_id,
        on_result);
    self::ton_client::destroy_context(context)
}

