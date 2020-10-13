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
use futures::{Future, SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use reqwest::{
    Client as HttpClient, ClientBuilder, Method,
    header::{HeaderMap, HeaderName, HeaderValue}
};
use crate::error::ClientResult;
use super::{ClientEnv, Error, WebSocket, FetchMethod, FetchResult};

pub(crate) struct ClientEnvImpl {
    http_client: HttpClient,
    _async_runtime: Option<tokio::runtime::Runtime>,
    async_runtime_handle: tokio::runtime::Handle,
}

impl ClientEnvImpl {
    pub fn new() -> ClientResult<Self> {
        let client = ClientBuilder::new()
            .build()
            .map_err(|err| Error::http_client_create_error(err))?;

        let (async_runtime, async_runtime_handle) =
            if let Ok(existing) = tokio::runtime::Handle::try_current() {
                (None, existing)
            } else {
                let runtime = tokio::runtime::Builder::new()
                    .threaded_scheduler()
                    .enable_io()
                    .enable_time()
                    .build()
                    .map_err(|err| Error::cannot_create_runtime(err))?;
                let runtime_handle = runtime.handle().clone();
                (Some(runtime), runtime_handle)
            };

        Ok(Self {
            http_client: client,
            _async_runtime: async_runtime,
            async_runtime_handle
        })
    }

    fn string_map_to_header_map(headers: HashMap<String, String>) -> ClientResult<HeaderMap> {
        let mut map = HeaderMap::new();
        for (key, value) in headers {
            let header_name = HeaderName::from_str(key.as_str())
                .map_err(|err| Error::http_request_create_error(err))?;
            let header_value = HeaderValue::from_str(value.as_str())
                .map_err(|err| Error::http_request_create_error(err))?;
            map.insert(header_name, header_value);
        }
        Ok(map)
    }

    fn header_map_to_string_map(headers: &HeaderMap) -> HashMap<String, String> {
        headers.into_iter()
            .filter_map(|(name, value)| {
                if let Ok(value) = value.to_str() {
                    Some((name.to_string(), value.to_string()))
                } else {
                    None
                }
            })
            .collect()
    }
}

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
    pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.async_runtime_handle.enter(move || 
            tokio::spawn(future)
        );
    }

    /// Executes asynchronous task blocking current thread
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.async_runtime_handle.block_on(future)
    }

    /// Connects to the websocket endpoint
    pub async fn websocket_connect(
        &self,
        url: &str,
        headers: Option<HashMap<&str, &str>>,
    ) -> ClientResult<WebSocket> {
        let mut request = tokio_tungstenite::tungstenite::handshake::client::Request::builder()
            .method("GET")
            .uri(url);

        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        let request = request
            .body(())
            .map_err(|err| Error::websocket_connect_error(url, err))?;

        let (client, _) = tokio_tungstenite::connect_async(request)
        .await
        .map_err(|err|
            Error::websocket_connect_error(url, err))?;

        let (write, read) = client.split();

        let write = write
            .sink_map_err(|err| Error::websocket_send_error(err))
            .with(|text| async move {
                Ok(WsMessage::text(text))
            });

        let read = read.filter_map(|result| async move {
            match result {
                Ok(message) => {
                    match message {
                        WsMessage::Text(text) => Some(Ok(text)),
                        _ => None
                    }
                },
                Err(err) => Some(Err(Error::websocket_receive_error(err))),
            }
        });

        Ok(WebSocket {
            receiver: Box::pin(read),
            sender: Box::pin(write),
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
        let method = Method::from_str(method.as_str())
            .map_err(|err| Error::http_request_create_error(err))?;

        let mut request = self.http_client.request(method, url);

        if let Some(headers) = headers {
            request = request.headers(Self::string_map_to_header_map(headers)?);
        }
        if let Some(body) = body {
            request = request.body(body);
        }
        if let Some(timeout) = timeout_ms {
            request = request.timeout(std::time::Duration::from_millis(timeout as u64));
        }

        let response = request.send()
            .await
            .map_err(|err| Error::http_request_send_error(err))?;

        Ok(FetchResult {
            headers: Self::header_map_to_string_map(response.headers()),
            status: response.status().as_u16(),
            url: response.url().to_string(),
            body: response.text()
                .await
                .map_err(|err| Error::http_request_parse_error(err))?
        })
    }
}
