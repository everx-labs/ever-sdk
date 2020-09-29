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
use crate::error::ApiResult;
use super::{ClientEnv, Error, WebSocket, FetchMethod, FetchResult};

pub(crate) struct StdClientEnv {
    http_client: HttpClient,
}

impl StdClientEnv {
    pub fn new() -> ApiResult<Self> {
        let client = ClientBuilder::new()  
            .build()
            .map_err(|err| Error::http_client_create_error(err))?;

        Ok(StdClientEnv {
            http_client: client
        })
    }

    fn string_map_to_header_map(headers: HashMap<String, String>) -> ApiResult<HeaderMap> {
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

#[async_trait::async_trait]
impl ClientEnv for StdClientEnv {
    /// Returns current Unix time in ms
    fn now_ms(&self) -> u64 {
        chrono::prelude::Utc::now().timestamp_millis() as u64
    }

    /// Sets timer for provided time interval
    async fn set_timer(&self, ms: u64) {
        tokio::time::delay_for(tokio::time::Duration::from_millis(ms)).await
    }

    /// Sends asynchronous task to scheduler
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
        tokio::spawn(future);
    }

    /// Connects to the websocket endpoint
    async fn websocket_connect(
        &self,
        url: &str,
        headers: Option<HashMap<&str, &str>>,
    ) -> ApiResult<WebSocket> {
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
    async fn websocket_close(&self, _handle: u32) {}

    /// Executes http request
    async fn fetch(
        &self,
        url: &str,
        method: FetchMethod,
        headers: Option<HashMap<String, String>>,
        body: Option<Vec<u8>>,
        timeout_ms: Option<u32>,
    ) -> ApiResult<FetchResult> {
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
            body: response.bytes()
                .await
                .map_err(|err| Error::http_request_parse_error(err))?
                .to_vec()
        })
    }
}