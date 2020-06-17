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

extern crate js_sys;
extern crate wasm_bindgen;
extern crate serde_json;
extern crate ton_client;

use wasm_bindgen::prelude::*;
use ton_client::{json_sync_request, create_context, destroy_context, JsonResponse};

fn to_json(response: &JsonResponse) -> String {
    serde_json::to_string(&response)
        .unwrap_or(r#"{
            "result_json": "",
            "error_json": { "message": "Can not convert response into JSON" }
        }"#.to_string())
}

#[wasm_bindgen]
pub fn core_request(context: u32, method: String, params_json: String) -> String {
    let response = json_sync_request(context, method, params_json);
    to_json(&response)
}

#[wasm_bindgen]
pub fn core_create_context() -> u32 {
    create_context()
}

#[wasm_bindgen]
pub fn core_destroy_context(context: u32) {
    destroy_context(context)
}

#[wasm_bindgen]
pub fn request(method: String, params_json: String) -> String {
    let context = create_context();
    let response = json_sync_request(context, method, params_json);
    destroy_context(context);
    to_json(&response)
}
