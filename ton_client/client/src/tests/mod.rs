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

mod test_misc;

use serde_json::{Value, Map};
use log::{Metadata, Record, LevelFilter};
use crate::{InteropContext, tc_create_context, tc_json_request, InteropString, tc_read_json_response, tc_destroy_json_response, tc_destroy_context};

pub const LOG_CGF_PATH: &str = "src/tests/log_cfg.yaml";

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        println!("{} - {}", record.level(), record.args());
    }

    fn flush(&self) {}
}

#[derive(Clone)]
pub(crate) struct TestClient {
    context: InteropContext,
}

impl TestClient {
    pub(crate) fn init_log() {
        let log_cfg_path = LOG_CGF_PATH;
        let _ = log4rs::init_file(log_cfg_path, Default::default());
    }

    pub(crate) fn missing_params() -> Value {
        Value::String(String::new())
    }

    pub(crate) fn get_network_address() -> String {
        std::env::var("TON_NETWORK_ADDRESS").unwrap_or("http://localhost:8080".to_owned())
    }

    pub(crate) fn new() -> Self {
        Self::new_with_config(json!({
            "baseUrl": Self::get_network_address()
        }))
    }

    pub(crate) fn new_with_config(config: Value) -> Self {
        let _ = log::set_boxed_logger(Box::new(SimpleLogger))
            .map(|()| log::set_max_level(LevelFilter::Debug));

        let context: InteropContext;
        unsafe {
            context = tc_create_context()
        }
        let client = Self { context };
        if config != Value::Null {
            client.request("setup", config).unwrap();
        }
        client
    }

    pub(crate) fn request(
        &self,
        method_name: &str,
        params: Value,
    ) -> Result<String, String> {
        unsafe {
            let params_json = if params.is_null() { String::new() } else { params.to_string() };
            let response_ptr = tc_json_request(
                self.context,
                InteropString::from(&method_name.to_string()),
                InteropString::from(&params_json),
            );
            let interop_response = tc_read_json_response(response_ptr);
            let response = interop_response.to_response();
            tc_destroy_json_response(response_ptr);
            if response.error_json.is_empty() {
                Ok(response.result_json)
            } else {
                Err(response.error_json)
            }
        }
    }

    pub(crate) fn request_map(
        &self,
        method_name: &str,
        params: Value,
    ) -> Map<String, Value> {
        Self::parse_object(self.request(method_name, params))
    }

    pub(crate) fn parse_object(s: Result<String, String>) -> Map<String, Value> {
        if let Value::Object(m) = serde_json::from_str(s.unwrap().as_str()).unwrap() {
            return m.clone();
        }
        panic!("Object expected");
    }

    pub(crate) fn parse_string(r: Result<String, String>) -> String {
        if let Value::String(s) = serde_json::from_str(r.unwrap().as_str()).unwrap() {
            return s.clone();
        }
        panic!("String expected");
    }

    pub(crate) fn get_map_string(m: &Map<String, Value>, f: &str) -> String {
        if let Value::String(s) = m.get(f).unwrap() {
            return s.clone();
        }
        panic!("Field not fount");
    }
}

impl Drop for TestClient {
    fn drop(&mut self) {
        unsafe {
            if self.context != 0 {
                tc_destroy_context(self.context)
            }
        }
    }
}
