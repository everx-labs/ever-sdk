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

use super::{Error, FetchMethod, FetchResult, WebSocket};
#[cfg(test)]
use crate::client::client_env::TestFetch;
use crate::error::ClientResult;
use futures::{Future, SinkExt, StreamExt};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client as HttpClient, ClientBuilder, Method,
};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::runtime::Runtime;
#[cfg(test)]
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message as WsMessage;

lazy_static! {
    static ref RUNTIME_CONTAINER: ClientResult<Runtime> = create_runtime();
}

fn create_runtime() -> ClientResult<Runtime> {
    tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|err| Error::cannot_create_runtime(err))
}

#[cfg(test)]
pub(crate) struct TestEnv {
    pub fetch_queue: Option<Vec<TestFetch>>,
}

#[cfg(test)]
impl TestEnv {
    fn new() -> Self {
        Self { fetch_queue: None }
    }

    #[cfg(test)]
    fn dequeue_fetch(&mut self, url: &str) -> Option<TestFetch> {
        fn same_endpoints(url1: &str, url2: &str) -> bool {
            fn reduce_url(url: &str) -> String {
                let mut url = url.to_lowercase();
                if let Some(without_protocol) = url.strip_prefix("http://") {
                    url = without_protocol.to_string();
                }
                if let Some(without_protocol) = url.strip_prefix("https://") {
                    url = without_protocol.to_string();
                }
                url
            }
            let a = reduce_url(url1);
            let b = reduce_url(url2);
            return a.starts_with(&b) || b.starts_with(&a);
        }
        if let Some(queue) = &mut self.fetch_queue {
            let next_index = queue.iter().position(|x| same_endpoints(&x.url, url));
            Some(match next_index {
                Some(index) => queue.remove(index),
                None => TestFetch {
                    delay: None,
                    url: url.to_string(),
                    result: Err(crate::client::Error::http_request_send_error(
                        "Test fetch queue is empty",
                    )),
                },
            })
        } else {
            None
        }
    }
}

pub(crate) struct ClientEnv {
    http_client: HttpClient,
    async_runtime_handle: tokio::runtime::Handle,
    #[cfg(test)]
    test: RwLock<TestEnv>,
}

impl ClientEnv {
    pub fn new() -> ClientResult<Self> {
        let client = ClientBuilder::new()
            .build()
            .map_err(|err| Error::http_client_create_error(err))?;

        let async_runtime_handle = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => RUNTIME_CONTAINER
                .as_ref()
                .map_err(|err| err.clone())?
                .handle()
                .clone(),
        };

        Ok(Self {
            http_client: client,
            async_runtime_handle,
            #[cfg(test)]
            test: RwLock::new(TestEnv::new()),
        })
    }

    #[cfg(test)]
    pub async fn set_test_fetch_queue(&self, queue: Option<Vec<TestFetch>>) {
        self.test.write().await.fetch_queue = queue;
    }

    #[cfg(test)]
    pub async fn get_test_fetch_queue(&self) -> Option<Vec<TestFetch>> {
        self.test.read().await.fetch_queue.clone()
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
        headers
            .into_iter()
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

impl ClientEnv {
    /// Returns current Unix time in ms
    pub fn now_ms(&self) -> u64 {
        chrono::prelude::Utc::now().timestamp_millis() as u64
    }

    /// Sets timer for provided time interval
    pub async fn set_timer(&self, ms: u64) -> ClientResult<()> {
        tokio::time::delay_for(tokio::time::Duration::from_millis(ms)).await;
        Ok(())
    }

    /// Sends asynchronous task to scheduler
    pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.async_runtime_handle
            .enter(move || tokio::spawn(future));
    }

    /// Executes asynchronous task blocking current thread
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.async_runtime_handle.block_on(future)
    }

    /// Connects to the websocket endpoint
    pub async fn websocket_connect(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> ClientResult<WebSocket> {
        let mut request = tokio_tungstenite::tungstenite::handshake::client::Request::builder()
            .method("GET")
            .uri(url);

        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(&key, &value);
            }
        }

        let request = request
            .body(())
            .map_err(|err| Error::websocket_connect_error(url, err))?;

        let (client, _) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(|err| Error::websocket_connect_error(url, err))?;

        let (write, read) = client.split();

        let write = write
            .sink_map_err(|err| Error::websocket_send_error(err))
            .with(|text| async move { Ok(WsMessage::text(text)) });

        let read = read.filter_map(|result| async move {
            match result {
                Ok(message) => match message {
                    WsMessage::Text(text) => Some(Ok(text)),
                    _ => None,
                },
                Err(err) => Some(Err(Error::websocket_receive_error(err))),
            }
        });

        Ok(WebSocket {
            receiver: Box::pin(read),
            sender: Box::pin(write),
        })
    }

    #[cfg(test)]
    async fn get_next_test_fetch(
        &self,
        url: &str,
        body: &Option<String>,
    ) -> Option<ClientResult<FetchResult>> {
        let fetch = { self.test.write().await.dequeue_fetch(url) };
        if let Some(fetch) = fetch {
            let delay_log = if let Some(delay) = fetch.delay {
                let _ = self.set_timer(delay).await;
                format!(" {} ms ", delay)
            } else {
                String::default()
            };
            let mut result = fetch.result;
            if let Ok(result) = &mut result {
                result.url = url.split("?").next().unwrap_or("").to_string();
            }
            let mut log = format!("Fetch {}", url);
            if let Some(body) = &body {
                log.push_str(&format!("\n  ⤷ {}", body));
            }
            match &result {
                Ok(ok) => log.push_str(
                    &format!("\n  {:?}", ok).replace("FetchResult", &format!("✅{}", delay_log)),
                ),
                Err(err) => log.push_str(
                    &format!("\n  {:?}", err).replace("ClientError", &format!("❌{}", delay_log)),
                ),
            };
            println!("{}", log);
            Some(result)
        } else {
            None
        }
    }

    /// Executes http request
    pub async fn fetch(
        &self,
        url: &str,
        method: FetchMethod,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
        timeout_ms: Option<u32>,
    ) -> ClientResult<FetchResult> {
        #[cfg(test)]
        {
            if let Some(result) = self.get_next_test_fetch(url, &body).await {
                return result;
            }
        }
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

        let response = request
            .send()
            .await
            .map_err(|err| Error::http_request_send_error(err))?;

        Ok(FetchResult {
            headers: Self::header_map_to_string_map(response.headers()),
            status: response.status().as_u16(),
            url: response.url().to_string(),
            body: response
                .text()
                .await
                .map_err(|err| Error::http_request_parse_error(err))?,
        })
    }
}
