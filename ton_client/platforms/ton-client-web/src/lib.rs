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

use ton_client::{create_context, destroy_context, request};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn core_create_context(config_json: String) -> String {
    create_context(config_json)
}

#[wasm_bindgen]
pub fn core_destroy_context(context: u32) {
    destroy_context(context)
}

#[wasm_bindgen(js_namespace = tonclient)]
extern "C" {
    fn core_response_handler(
        request_id: u32,
        params_json: String,
        response_type: u32,
        finished: bool,
    );
}

fn response_handler(request_id: u32, params_json: String, response_type: u32, finished: bool) {
    core_response_handler(request_id, params_json, response_type, finished);
}

#[wasm_bindgen]
pub fn core_request(context: u32, function_name: String, params_json: String, request_id: u32) {
    request(
        context,
        function_name,
        params_json,
        request_id,
        response_handler,
    );
}
