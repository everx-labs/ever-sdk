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

use std::str::FromStr;
use std::collections::HashMap;
use std::pin::Pin;
use futures::{Future, FutureExt, SinkExt, StreamExt};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use crate::error::ClientResult;
use super::{ClientEnv, Error, WebSocket, FetchMethod, FetchResult};

pub(crate) struct ClientEnvImpl {}

impl ClientEnvImpl {
    pub fn new() -> ClientResult<Self> {
        Ok(Self {})
    }

    /// Executes http request
    async fn fetch_internal(
        url: &str,
        method: FetchMethod,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
        _timeout_ms: Option<u32>,
    ) -> ClientResult<FetchResult> {
        let mut opts = RequestInit::new();
        opts.method(method.as_str());

        if let Some(body) = body {
            opts.body(Some(&JsValue::from_str(&body)));
        }

        let request = Request::new_with_str_and_init(url, &opts)
            .map_err(|_| Error::http_request_create_error("Can not create request"))?;

        if let Some(headers) = headers {
            let request_headers = request.headers();
            for (key, value) in headers {
                request_headers.set(&key, &value)
                    .map_err(|_| Error::http_request_create_error("Can not set header value"))?;
            }
        }
        // TODO: set request timeout

        let window = web_sys::window()
            .ok_or_else(|| Error::http_request_create_error("Can not get `window`"))?;

        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .map(|result| {
                match result {
                    Ok(result) => Ok(result),
                    Err(err) => {
                        let err = err.into_serde::<serde_json::Value>()
                            .map(|val| format!("{:#}", val))
                            .unwrap_or("Unserializable error".to_owned());
                        Err(Error::http_request_send_error(err))
                    }
                }
            })
            .await?;

        let response: Response = resp_value.dyn_into()
            .map_err(|_| 
                Error::http_request_parse_error("Can not cast response to `Response` struct"))?;

        let text = JsFuture::from(
            response.text()
                .map_err(|_| Error::http_request_parse_error("Can not get text from response"))?
        )
            .await
            .map_err(|_| Error::http_request_parse_error("Response body is not a text"))?
            .as_string()
            .ok_or_else(|| Error::http_request_parse_error("Answer value is not a string"))?;

        Ok(FetchResult {
            // TODO: extract headers
            headers: HashMap::new(),
            status: response.status(),
            url: response.url().to_string(),
            body: text
        })
    }
}

//#[async_trait::async_trait]
impl ClientEnvImpl {
    /// Returns current Unix time in ms
    pub fn now_ms(&self) -> u64 {
        chrono::prelude::Utc::now().timestamp_millis() as u64
    }

    /// Sets timer for provided time interval
    pub async fn set_timer(&self, ms: u64) {
        tokio::time::delay_for(tokio::time::Duration::from_millis(ms)).await
    }

    /// Sends asynchronous task to scheduler
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(future);
    }

    /// Connects to the websocket endpoint
    pub async fn websocket_connect(
        &self,
        url: &str,
        headers: Option<HashMap<&str, &str>>,
    ) -> ClientResult<WebSocket> {
        
        Ok(WebSocket {
            receiver: Box::pin(futures::stream::empty()),
            sender: Box::pin(futures::sink::drain().sink_map_err(|_| Error::websocket_send_error(""))),
            handle: 0
        })
    }

    /// Closes websocket
    pub async fn websocket_close(&self, _handle: u32) {}

    /// Executes http request
    pub async fn fetch(
        &self,
        url: &str,
        method: FetchMethod,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
        timeout_ms: Option<u32>,
    ) -> ClientResult<FetchResult> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let url = url.to_owned();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = sender.send(
                Self::fetch_internal(&url, method, headers, body, timeout_ms).await
            );
        });

        receiver
            .await
            .map_err(|err| Error::http_request_parse_error(
                format!("cannot receive result from fetch task: {}", err)))?
    }
}