/*
* Copyright 2018-2021 TON Labs LTD.
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
use crate::client::LOCAL_STORAGE_DEFAULT_DIR_NAME;
use crate::client::storage::KeyValueStorage;
use crate::error::ClientResult;
use futures::{Future, FutureExt, SinkExt, StreamExt};
use indexed_db_futures::{IdbDatabase, IdbQuerySource, IdbVersionChangeEvent};
use indexed_db_futures::request::IdbOpenDbRequestLike;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Event, MessageEvent, Request, RequestInit, Response, Window, IdbTransactionMode};
use js_sys::JSON;

#[cfg(test)]
#[path = "client_env_tests.rs"]
mod client_env_tests;

fn js_error_to_string(js_value: JsValue) -> String {
    if let Ok(txt) = js_value.clone().dyn_into::<js_sys::Error>() {
        String::from(txt.message())
    } else {
        JSON::stringify(&js_value)
            .map(|val| String::from(val))
            .unwrap_or("Unserializable value".to_owned())
    }
}

fn js_value_to_string(js_value: &JsValue) -> ClientResult<String> {
    if let Ok(txt) = js_value.clone().dyn_into::<js_sys::JsString>() {
        Ok(String::from(txt))
    } else {
        JSON::stringify(js_value)
            .map(|val| String::from(val))
            .map_err(|_err| Error::internal_error("Cannot serialize JsValue"))
    }
}

// web-sys and wasm-bindgen types are not `Send` so we cannot use them directly in async
// functions which are registered in dispatcher: registration requires `Future` returned
// by the function to be `Send`, but using not `Send` types inside prevents it. So we have
// to process functions in another task, which encapsulates work with non-`Send` types
async fn execute_spawned<R, Fut, F>(func: F) -> ClientResult<R>
where
    R: 'static,
    Fut: Future<Output = R> + 'static,
    F: FnOnce() -> Fut + 'static,
{
    let (sender, receiver) = tokio::sync::oneshot::channel();

    wasm_bindgen_futures::spawn_local(async move {
        let _ = sender.send(func().await);
    });

    receiver
        .await
        .map_err(|err| Error::can_not_receive_spawned_result(err))
}

struct Timer{
    window: Window,
    timer_id: Option<i32>,
    // keep closure to fix memory leak
    _on_timer: Closure<dyn FnMut() -> ()>,
}

impl Timer {
    pub fn new(timeout_ms: u64) -> ClientResult<(Self, impl Future<Output=ClientResult<()>>)> {
        let window =
            web_sys::window().ok_or_else(|| Error::set_timer_error("Can not get `window`"))?;

        let (sender, receiver) = tokio::sync::oneshot::channel();
        let on_timer = Closure::once(move || {
            let _ = sender.send(());
        });

        let timer_id = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                on_timer.as_ref().unchecked_ref(),
                std::cmp::min(timeout_ms, std::i32::MAX as u64) as i32,
            )
            .map_err(|_| Error::set_timer_error("Can not set timer"))?;

        Ok((
            Self {
                window,
                timer_id: Some(timer_id),
                _on_timer: on_timer,
            },
            receiver
                .map(|val| val.map_err(|_| Error::set_timer_error("Can not receive timer result")))
        ))
    }

    pub fn forget(&mut self) {
        self.timer_id = None;
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if let Some(timer_id) = self.timer_id {
            self.window.clear_timeout_with_handle(timer_id);
        }
    }
}

pub(crate) struct ClientEnv {}

impl ClientEnv {
    pub fn new() -> ClientResult<Self> {
        Ok(Self {})
    }

    /// Sets timer for provided time interval
    pub async fn set_timer_internal(ms: u64) -> ClientResult<()> {
        let (mut timer, future) = Timer::new(ms)?;
        future.await?;
        Ok(timer.forget())
    }

    /// Connects to the websocket endpoint
    pub async fn websocket_connect_internal(
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> ClientResult<WebSocket> {
        // Connect to a server
        let ws = if let Some(Some(protocols)) =
            headers.map(|mut headers| headers.remove("Sec-WebSocket-Protocol"))
        {
            web_sys::WebSocket::new_with_str(url, &protocols)
        } else {
            web_sys::WebSocket::new(url)
        }
        .map_err(|_| Error::websocket_connect_error(url, "cannot create websocket"))?;

        let (on_message_sink, on_message_stream) = futures::channel::mpsc::channel(100);
        let on_message = move |result: ClientResult<String>| {
            let mut on_message_sink = on_message_sink.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = on_message_sink.send(result).await;
            });
        };
        let on_error = on_message.clone();

        // create callback
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            // process only text messages
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let string = String::from(txt);
                log::trace!("Websocket received {}", string);
                on_message(Ok(string));
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        // set message event handler on WebSocket
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        // forget the callback to keep it alive
        onmessage_callback.forget();

        // on_open callback to get notification when websocket is ready to use
        let (mut on_open_sender, mut on_open_receiver) = tokio::sync::mpsc::channel(1);
        let mut on_open_sender_copy = on_open_sender.clone();
        let onopen_callback = Closure::once(move |_: JsValue| {
            log::trace!("Websocket opened");
            wasm_bindgen_futures::spawn_local(async move {
                let _ = on_open_sender_copy.send(Ok(())).await;
            });
        });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // initialization errors handling callback
        let onerror_callback = Closure::once(move |e: Event| {
            log::debug!("Websocket init error {:#?}", e);
            wasm_bindgen_futures::spawn_local(async move {
                let _ = on_open_sender.send(Err(())).await;
            });
        });
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // wait for websocket opening or error occurred during initialization
        on_open_receiver
            .recv()
            .await
            .ok_or_else(|| {
                let _ = ws.close();
                Error::websocket_connect_error(url, "can not receive websocket init result")
            })?
            .map_err(|_| {
                let _ = ws.close();
                Error::websocket_connect_error(url, "can not open websocket")
            })?;

        // change error handler to send errors to output stream
        let onerror_callback = Closure::wrap(Box::new(move |e: Event| {
            log::debug!("Websocket error {:#?}", e);
            on_error(Err(Error::websocket_receive_error("")));
        }) as Box<dyn FnMut(Event)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // sending messages in another task to encapsulate non-`Send` `WebSocket` instance there
        let (send_sink, mut send_stream) = futures::channel::mpsc::channel::<(
            String,
            tokio::sync::oneshot::Sender<ClientResult<()>>,
        )>(100);
        wasm_bindgen_futures::spawn_local(async move {
            log::trace!("Start websocket sending loop");
            while let Some((string, sender)) = send_stream.next().await {
                if string.is_empty() {
                    break;
                }
                log::trace!("Websocket send: {}", string);
                let result = ws
                    .send_with_str(&string)
                    .map_err(|err| Error::websocket_send_error(js_error_to_string(err)));
                let _ = sender.send(result);
            }
            let _ = ws.close();
            log::trace!("End websocket sending loop");
        });

        // to check result of sending message to websocket we send string via channel to spawned
        // sending task and then wait for the result from oneshot channel
        let send_sink = futures::sink::drain()
            .sink_map_err(|err| Error::websocket_send_error(err))
            .with(move |string: String| {
                let mut send_sink = send_sink.clone();
                async move {
                    let (sender, receiver) = tokio::sync::oneshot::channel();
                    // send string along with oneshot sender which will receive sending result
                    send_sink.send((string, sender)).await.map_err(|err| {
                        Error::websocket_send_error(format!(
                            "can not send data to websocket sending task: {}",
                            err
                        ))
                    })?;
                    // receive sending result
                    receiver.await.map_err(|err| {
                        Error::websocket_send_error(format!(
                            "can not receive result from websocket sending task: {}",
                            err
                        ))
                    })?
                }
            });

        Ok(WebSocket {
            receiver: Box::pin(on_message_stream),
            sender: Box::pin(send_sink),
        })
    }

    /// Executes http request
    async fn fetch_internal(
        url: &str,
        method: FetchMethod,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
        timeout_ms: Option<u32>,
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
                request_headers
                    .set(&key, &value)
                    .map_err(|_| Error::http_request_create_error("Can not set header value"))?;
            }
        }
        // TODO: set request timeout
        let window = web_sys::window()
            .ok_or_else(|| Error::http_request_create_error("Can not get `window`"))?;

        let mut resp_future = JsFuture::from(window.fetch_with_request(&request))
            .map(|result| match result {
                Ok(result) => Ok(result),
                Err(err) => Err(Error::http_request_send_error(js_error_to_string(err))),
            });

        let resp_result = match timeout_ms {
            Some(timeout) => {
                futures::select!(
                    result = resp_future => result,
                    timer = Self::set_timer_internal(timeout as u64).fuse() => {
                        Err(timer
                            .err()
                            .unwrap_or(Error::http_request_send_error("fetch operation timeout")))
                    }
                )
            }
            None => resp_future.await
        };

        let response: Response = resp_result?.dyn_into().map_err(|_| {
            Error::http_request_parse_error("Can not cast response to `Response` struct")
        })?;

        let text = JsFuture::from(
            response
                .text()
                .map_err(|_| Error::http_request_parse_error("Can not get text from response"))?,
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
            body: text,
            remote_address: None,
        })
    }
}

impl ClientEnv {
    /// Returns current Unix time in ms
    pub fn now_ms(&self) -> u64 {
        chrono::prelude::Utc::now().timestamp_millis() as u64
    }

    /// Sets timer for provided time interval
    pub async fn set_timer(&self, ms: u64) -> ClientResult<()> {
        execute_spawned(move || Self::set_timer_internal(ms)).await?
    }

    /// Sends asynchronous task to scheduler
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(future);
    }

    /// Connects to the websocket endpoint
    pub async fn websocket_connect(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> ClientResult<WebSocket> {
        let url = url.to_owned();
        execute_spawned(
            move || async move { Self::websocket_connect_internal(&url, headers).await },
        )
        .await?
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
        let url = url.to_owned();
        execute_spawned(move || async move {
            Self::fetch_internal(&url, method, headers, body, timeout_ms).await
        })
        .await?
    }
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
        Ok(Self {
            local_storage_path,
            storage_name,
        })
    }

    async fn open_db(
        local_storage_path: &Option<String>,
        storage_name: &str,
    ) -> ClientResult<IdbDatabase> {
        let db_name = local_storage_path
            .as_deref()
            .unwrap_or(LOCAL_STORAGE_DEFAULT_DIR_NAME);

        let mut db_request = IdbDatabase::open(db_name)
            .map_err(|err| Error::local_storage_error(err.message()))?;

        let storage_name = storage_name.to_owned();
        db_request.set_on_upgrade_needed(Some(move |event: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            if event.db().object_store_names().find(|name| *name == storage_name).is_none() {
                event.db().create_object_store(&storage_name)?;
            }
            Ok(())
        }));

        db_request.into_future().await
            .map_err(|err| Error::local_storage_error(err.message()))
    }

    async fn read_bin(
        local_storage_path: &Option<String>,
        storage_name: &str,
        key: &str,
    ) -> ClientResult<Option<Vec<u8>>> {
        Ok(Self::read_str(local_storage_path, storage_name, key).await?
            .map(|content_base64| base64::decode(&content_base64))
            .transpose()
            .map_err(|err| Error::local_storage_error(err))?)
    }

    async fn write_bin(
        local_storage_path: &Option<String>,
        storage_name: &str,
        key: &str,
        value: &[u8],
    ) -> ClientResult<()> {
        Self::write_str(local_storage_path, storage_name, key, &base64::encode(value)).await
    }

    async fn read_str(
        local_storage_path: &Option<String>,
        storage_name: &str,
        key: &str,
    ) -> ClientResult<Option<String>> {
        let db = Self::open_db(local_storage_path, storage_name).await?;

        let tx = db.transaction_on_one_with_mode(storage_name, IdbTransactionMode::Readonly)
            .map_err(|err| Error::local_storage_error(err.message()))?;

        let store = tx.object_store(storage_name)
            .map_err(|err| Error::local_storage_error(err.message()))?;

        store.get(&JsValue::from_str(key))
            .map_err(|err| Error::local_storage_error(err.message()))?
            .await
            .map_err(|err| Error::local_storage_error(err.message()))?
            .map(|js_value| js_value_to_string(&js_value))
            .transpose()
    }

    async fn write_str(
        local_storage_path: &Option<String>,
        storage_name: &str,
        key: &str,
        value: &str,
    ) -> ClientResult<()> {
        let db = Self::open_db(local_storage_path, storage_name).await?;

        let tx = db.transaction_on_one_with_mode(storage_name, IdbTransactionMode::Readwrite)
            .map_err(|err| Error::local_storage_error(err.message()))?;

        let store = tx.object_store(storage_name)
            .map_err(|err| Error::local_storage_error(err.message()))?;

        store.put_key_val(&JsValue::from_str(key), &JsValue::from_str(value))
            .map_err(|err| Error::local_storage_error(err.message()))?
            .into_future()
            .await
            .map_err(|err| Error::local_storage_error(err.message()))
    }

    async fn remove_internal(
        local_storage_path: &Option<String>,
        storage_name: &str,
        key: &str,
    ) -> ClientResult<()> {
        let db = Self::open_db(local_storage_path, storage_name).await?;

        let tx = db.transaction_on_one_with_mode(storage_name, IdbTransactionMode::Readwrite)
            .map_err(|err| Error::local_storage_error(err.message()))?;

        let store = tx.object_store(storage_name)
            .map_err(|err| Error::local_storage_error(err.message()))?;

        store.delete(&JsValue::from_str(key))
            .map_err(|err| Error::local_storage_error(err.message()))?
            .into_future()
            .await
            .map_err(|err| Error::local_storage_error(err.message()))
    }
}

#[async_trait::async_trait]
impl KeyValueStorage for LocalStorage {
    async fn get_bin(&self, key: &str) -> ClientResult<Option<Vec<u8>>> {
        let local_storage_path = self.local_storage_path.clone();
        let storage_name = self.storage_name.clone();
        let key = key.to_owned();
        execute_spawned(
            move || async move {
                Self::read_bin(&local_storage_path, &storage_name, &key).await
            }
        ).await?
    }

    async fn put_bin(&self, key: &str, value: &[u8]) -> ClientResult<()> {
        let local_storage_path = self.local_storage_path.clone();
        let storage_name = self.storage_name.clone();
        let key = key.to_owned();
        let value = value.to_owned();
        execute_spawned(
            move || async move {
                Self::write_bin(&local_storage_path, &storage_name, &key, &value).await
            }
        ).await?
    }

    async fn get_str(&self, key: &str) -> ClientResult<Option<String>> {
        let local_storage_path = self.local_storage_path.clone();
        let storage_name = self.storage_name.clone();
        let key = key.to_owned();
        execute_spawned(
            move || async move {
                Self::read_str(&local_storage_path, &storage_name, &key).await
            }
        ).await?
    }

    async fn put_str(&self, key: &str, value: &str) -> ClientResult<()> {
        let local_storage_path = self.local_storage_path.clone();
        let storage_name = self.storage_name.clone();
        let key = key.to_owned();
        let value = value.to_owned();
        execute_spawned(
            move || async move {
                Self::write_str(&local_storage_path, &storage_name, &key, &value).await
            }
        ).await?
    }

    async fn remove(&self, key: &str) -> ClientResult<()> {
        let local_storage_path = self.local_storage_path.clone();
        let storage_name = self.storage_name.clone();
        let key = key.to_owned();
        execute_spawned(
            move || async move {
                Self::remove_internal(&local_storage_path, &storage_name, &key).await
            }
        ).await?
    }
}

