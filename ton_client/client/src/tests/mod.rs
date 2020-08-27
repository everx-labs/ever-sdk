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
use serde::{Serialize};
use crate::error::{ApiError, ApiResult};
use serde::de::DeserializeOwned;

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
            client.request_json("setup", config).unwrap();
        }
        client
    }

    pub(crate) fn request_json(&self, method: &str, params: Value) -> ApiResult<Value> {
        let response = unsafe {
            let params_json = if params.is_null() { String::new() } else { params.to_string() };
            let response_ptr = tc_json_request(
                self.context,
                InteropString::from(&method.to_string()),
                InteropString::from(&params_json),
            );
            let interop_response = tc_read_json_response(response_ptr);
            let response = interop_response.to_response();
            tc_destroy_json_response(response_ptr);
            response
        };
        if response.error_json.is_empty() {
            if response.result_json.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(serde_json::from_str(&response.result_json).unwrap())
            }
        } else {
            Err(serde_json::from_str(&response.error_json).unwrap())
        }
    }

    pub(crate) fn request<P, R>(&self, method: &str, params: P) -> R
        where P: Serialize, R: DeserializeOwned {
        let params = serde_json::to_value(params)
            .map_err(|err| ApiError::invalid_params("", err)).unwrap();
        let result = self.request_json(method, params).unwrap();
        serde_json::from_value(result)
            .map_err(|err| ApiError::invalid_params("", err))
            .unwrap()
    }

    pub(crate) fn request_no_params<R: DeserializeOwned>(&self, method: &str) -> R {
        let result = self.request_json(method, Value::Null).unwrap();
        serde_json::from_value(result)
            .map_err(|err| ApiError::invalid_params("", err))
            .unwrap()
    }

    pub(crate) fn request_map<P: Serialize>(&self, method_name: &str, params: P) -> Map<String, Value> {
        let params = serde_json::to_value(params)
            .map_err(|err| ApiError::invalid_params("", err)).unwrap();
        Self::parse_object(self.request_json(method_name, params))
    }

    pub(crate) fn parse_object(s: ApiResult<Value>) -> Map<String, Value> {
        s.unwrap().as_object().unwrap().clone()
    }

    pub(crate) fn parse_string(r: ApiResult<Value>) -> String {
        r.unwrap().as_str().unwrap().into()
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
