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

use crate::*;
use std::sync::Mutex;

use self::reqwest::Client;
use self::reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::json;
use std::io::Read;

lazy_static! {
    static ref CLIENT: Mutex<Option<(Client, RequestsConfig)>> = Mutex::new(None);
}

// Globally initializes client with server address
pub fn init(config: RequestsConfig) {
    let mut client = CLIENT.lock().unwrap();
    *client = Some((Client::new(), config));
}

pub fn uninit() {
    let mut client = CLIENT.lock().unwrap();
    *client = None;
}

// Sends message to node
pub fn send_message(key: &[u8], value: &[u8]) -> SdkResult<()> {
    if let Some((client, config)) = CLIENT.lock().unwrap().as_ref() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let key_encoded = base64::encode(key);
        let value_encoded = base64::encode(value);
        let body = json!({
            "records": [{ "key": key_encoded, "value": value_encoded }]
        });

        let result = client.post(&config.requests_server)
            .headers(headers)
            .body(body.to_string())
            .send();

        match result {
            Ok(result) => {
                if result.status().is_success() {
                    Ok(())
                } else {
                    let bytes: Vec<u8> = result.bytes().map(|b| if let Ok(b) = b { b } else { 0 }).collect();
                    let text = match String::from_utf8(bytes.clone()) {
                        Ok(text) => text,
                        Err(_) => hex::encode(bytes)
                    };
                    bail!(SdkErrorKind::InternalError(format!("Request failed: {}", text)))
                }
            }
            Err(err) => bail!(SdkErrorKind::InternalError(format!("Can not send request: {}", err)))
        }
    } else {
        bail!(SdkErrorKind::NotInitialized);
    }
}
