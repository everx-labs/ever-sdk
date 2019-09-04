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
