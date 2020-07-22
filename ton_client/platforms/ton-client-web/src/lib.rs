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

use wasm_bindgen::prelude::*;
use ton_client::{json_sync_request, create_context, destroy_context};

const INVALID_RESPONSE: &str = r#"{
    "result_json": "",
    "error_json": { "message": "Can not convert response into JSON" }
}"#;


#[wasm_bindgen]
pub fn core_create_context() -> u32 {
    create_context()
}

#[wasm_bindgen]
pub fn core_destroy_context(context: u32) {
    destroy_context(context)
}

#[wasm_bindgen]
pub fn core_json_request(context: u32, method: String, params_json: String) -> String {
    let response = json_sync_request(context, method, params_json);
    serde_json::to_string(&response).unwrap_or(INVALID_RESPONSE.to_string())
}
