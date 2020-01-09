/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
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
use ton_client::{json_sync_request, create_context};

#[wasm_bindgen]
pub fn request(method: String, params_json: String) -> String {
    serde_json::to_string(
        &json_sync_request(
            create_context(),
            method,
            params_json))
        .unwrap_or("{message: \"???\"".to_string())
}
