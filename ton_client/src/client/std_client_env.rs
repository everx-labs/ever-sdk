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
use crate::client::{LOCAL_STORAGE_DEFAULT_DIR_NAME};
#[cfg(test)]
use crate::client::network_mock::NetworkMock;
use crate::error::ClientResult;
use crate::utils::storage::KeyValueStorage;
use futures::{Future, SinkExt, StreamExt};
use lazy_static::lazy_static;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client as HttpClient, ClientBuilder, Method,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::runtime::Runtime;
#[cfg(test)]
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message as WsMessage;

#[cfg(test)]
#[path = "client_env_tests.rs"]
mod client_env_tests;

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

pub(crate) struct ClientEnv {
    http_client: HttpClient,
    async_runtime_handle: tokio::runtime::Handle,
    #[cfg(test)]
    pub network_mock: RwLock<NetworkMock>,
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
            network_mock: RwLock::new(NetworkMock::new()),
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
        #[cfg(test)]
        {
            if let Some(ws) = self
                .network_mock
                .write()
                .await
                .websocket_connect(&self.async_runtime_handle, url)
                .await
            {
                return Ok(ws);
            }
        }
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
            let fetch_mock = { self.network_mock.write().await.dequeue_fetch(url, &body) };
            if let Some(fetch) = fetch_mock {
                return fetch.get_result(&self, url).await;
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
            remote_address: response.remote_addr().map(|x| x.to_string()),
            body: response
                .text()
                .await
                .map_err(|err| Error::http_request_parse_error(err))?,
        })
    }
}

lazy_static! {
    static ref KEY_FORMAT_RE: regex::Regex = regex::Regex::new(r#"^[a-zA-Z0-9_\.]+?$"#).unwrap();
}

pub(crate) struct LocalStorage {
    local_storage_path: Option<String>,
    storage_name: String,
}

impl LocalStorage {
    pub async fn new(
        local_storage_path: Option<String>,
        storage_name: String,
    ) -> ClientResult<Self> {
        tokio::fs::create_dir_all(Self::calc_storage_path(&local_storage_path, &storage_name)).await
            .map_err(|err| Error::local_storage_error(err))?;

        Ok(Self {
            local_storage_path,
            storage_name,
        })
    }

    fn calc_storage_path(
        local_storage_path: &Option<String>,
        storage_name: &str,
    ) -> PathBuf {
        let local_storage_path = local_storage_path
            .clone()
            .map(|path| PathBuf::from(path))
            .unwrap_or_else(|| {
                home::home_dir()
                    .unwrap_or(PathBuf::from("/"))
                    .join(LOCAL_STORAGE_DEFAULT_DIR_NAME)
            });

        local_storage_path
            .join(storage_name)
    }

    pub fn is_storage_key_correct(key: &str) -> bool {
        KEY_FORMAT_RE.is_match(key)
    }

    fn key_to_path(&self, key: &str) -> ClientResult<PathBuf> {
        if !Self::is_storage_key_correct(key) {
            return Err(Error::invalid_storage_key(key));
        }

        Ok(
            Self::calc_storage_path(&self.local_storage_path, &self.storage_name)
                .join(key)
        )
    }
}

#[async_trait::async_trait]
impl KeyValueStorage for LocalStorage {
    /// Get binary value by a given key from the storage
    async fn get_bin(&self, key: &str) -> ClientResult<Option<Vec<u8>>> {
        let path = self.key_to_path(key)?;

        match tokio::fs::read(&path).await {
            Ok(value) => Ok(Some(value)),
            Err(err) => if err.kind() == std::io::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(Error::local_storage_error(err))
            }
        }
    }

    /// Put binary value by a given key into the storage
    async fn put_bin(&self, key: &str, value: &[u8]) -> ClientResult<()> {
        let path = self.key_to_path(key)?;

        tokio::fs::write(&path, value).await
            .map_err(|err| Error::local_storage_error(err))
    }

    /// Get string value by a given key from the storage
    async fn get_str(&self, key: &str) -> ClientResult<Option<String>> {
        self.get_bin(key).await
            .map(|opt| opt.map(|vec| String::from_utf8(vec)))?
            .transpose()
            .map_err(|err| Error::local_storage_error(err))
            .map_err(|err| err.into())
    }

    /// Put string value by a given key into the storage
    async fn put_str(&self, key: &str, value: &str) -> ClientResult<()> {
        self.put_bin(key, value.as_bytes()).await
    }

    /// Remove value by a given key
    async fn remove(&self, key: &str) -> ClientResult<()> {
        let path = self.key_to_path(key)?;

        tokio::fs::remove_file(&path).await
            .map_err(|err| Error::local_storage_error(err))
    }
}
