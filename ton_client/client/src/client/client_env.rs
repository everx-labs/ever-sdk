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

use crate::error::{ClientError, ClientResult};
use super::Error;
use std::collections::HashMap;
use std::pin::Pin;
use futures::{Future, Sink, Stream};

pub(crate) struct WebSocket {
    pub handle: u32,
    pub sender: Pin<Box<dyn Sink<String, Error=ClientError> + Send>>,
    pub receiver: Pin<Box<dyn Stream<Item=ClientResult<String>> + Send>>
}

#[derive(Debug)]
pub(crate) struct FetchResult {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub url: String
}

impl FetchResult {
    pub fn body_as_text(&self) -> ClientResult<&str> {
        Ok(&self.body)
    }

    pub fn body_as_json(&self) -> ClientResult<serde_json::Value> {
        let text = self.body_as_text()?;
        serde_json::from_str(text)
            .map_err(|err| Error::http_request_parse_error(
                format!("Body is not a valid JSON: {}\n{}", err, text)))
    }
}

#[allow(dead_code)]
pub(crate) enum FetchMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Patch,
    Trace,
}

impl FetchMethod {
    pub fn as_str(&self) -> &str {
        match self {
            FetchMethod::Get => "GET",
            FetchMethod::Post => "POST",
            FetchMethod::Put => "PUT",
            FetchMethod::Delete => "DELETE",
            FetchMethod::Head => "HEAD",
            FetchMethod::Options => "OPTIONS",
            FetchMethod::Connect => "CONNECT",
            FetchMethod::Patch => "PATCH",
            FetchMethod::Trace => "TRACE",
        }
    }
}

#[async_trait::async_trait]
pub(crate) trait ClientEnv {
    /// Returns current Unix time in ms
    fn now_ms(&self) -> u64;
    /// Sets timer for provided time interval
    async fn set_timer(&self, ms: u64);
    /// Sends asynchronous task to scheduler
    fn spawn(&self, future: impl Future<Output = ()> + 'static);
    /// Executes asynchronous task blocking current thread
    #[cfg(not(target_arch = "wasm32"))]
    fn block_on<F: Future>(&self, future: F) -> F::Output;
    /// Connects to the websocket endpoint
    async fn websocket_connect(
        &self,
        url: &str,
        headers: Option<HashMap<&str, &str>>,
    ) -> ClientResult<WebSocket>;
    /// Closes websocket
    async fn websocket_close(&self, handle: u32);
    /// Executes http request
    async fn fetch(
        &self,
        url: &str,
        method: FetchMethod,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
        timeout_ms: Option<u32>,
    ) -> ClientResult<FetchResult>;
}
