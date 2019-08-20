use crate::*;
use std::sync::Mutex;

extern crate reqwest;
extern crate base64;

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

        let now = std::time::Instant::now();

        let result = client.post(&config.requests_server)
            .headers(headers)
            .body(body.to_string())
            .send();

        let t = now.elapsed();
	    println!("send time: sec={}.{:06} ", t.as_secs(), t.subsec_micros());

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
