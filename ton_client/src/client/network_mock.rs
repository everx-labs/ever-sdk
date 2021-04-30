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
 *
 */

use crate::client::{FetchResult, WebSocket};
use crate::error::ClientResult;
use futures::SinkExt;
#[derive(Debug, Clone)]
pub(crate) struct FetchMock {
    pub id: usize,
    pub url: String,
    pub delay: Option<u64>,
    pub result: ClientResult<FetchResult>,
}

impl FetchMock {
    pub async fn get_result(
        self,
        env: &crate::client::ClientEnv,
        url: &str,
    ) -> ClientResult<FetchResult> {
        if let Some(delay) = self.delay {
            let _ = env.set_timer(delay).await;
        }
        let mut result = self.result;
        let id = if self.id != 0 {
            format!(" {}", self.id)
        } else {
            String::default()
        };
        if let Ok(result) = &mut result {
            result.url = url.split("?").next().unwrap_or("").to_string();
        }
        let (text, find, replace_with) = match &result {
            Ok(ok) => (format!("  {:?}", ok), "FetchResult", "✅"),
            Err(err) => (format!("  {:?}", err), "ClientError", "❌"),
        };
        println!("{}", text.replace(find, &format!("{}{}", replace_with, id)));
        result
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MessageMock {
    pub url: String,
    pub delay: Option<u64>,
    pub message: String,
}

pub(crate) struct NetworkMock {
    pub fetches: Option<Vec<FetchMock>>,
    pub messages: Option<Vec<MessageMock>>,
}

fn same_endpoints(url1: &str, url2: &str) -> bool {
    fn reduce_url(url: &str) -> String {
        url.split("://").last().unwrap_or(url).to_lowercase()
    }
    let a = reduce_url(url1);
    let b = reduce_url(url2);
    return a.starts_with(&b) || b.starts_with(&a);
}

impl NetworkMock {
    pub(crate) fn new() -> Self {
        Self {
            fetches: None,
            messages: None,
        }
    }

    fn extract_messages(&mut self, url: &str) -> Vec<MessageMock> {
        let mut result = Vec::new();
        if let Some(messages) = &mut self.messages {
            let mut i = 0;
            while i < messages.len() {
                if same_endpoints(url, &messages[i].url) {
                    result.push(messages.remove(i));
                } else {
                    i += 1;
                }
            }
        }
        result
    }

    pub async fn websocket_connect(
        &mut self,
        async_runtime_handle: &tokio::runtime::Handle,
        url: &str,
    ) -> Option<WebSocket> {
        let mut messages = self.extract_messages(url);
        if messages.len() > 0 {
            let (client_sender, server_receiver) = futures::channel::mpsc::channel::<String>(10);
            let (mut server_sender, client_receiver) =
                futures::channel::mpsc::channel::<ClientResult<String>>(10);
            async_runtime_handle.enter(move || {
                tokio::spawn(Box::pin(async move {
                    let _ = server_receiver;
                    while !messages.is_empty() {
                        let message = messages.remove(0);
                        println!("Send {}", message.message);
                        if let Some(delay) = message.delay {
                            tokio::time::delay_for(tokio::time::Duration::from_millis(delay)).await;
                        }
                        let _ = server_sender.send(Ok(message.message)).await;
                    }
                }))
            });
            Some(WebSocket {
                receiver: Box::pin(client_receiver),
                sender: Box::pin(
                    client_sender
                        .sink_map_err(|err| crate::client::Error::websocket_send_error(err)),
                ),
            })
        } else {
            None
        }
    }

    pub(crate) fn dequeue_fetch(&mut self, url: &str, body: &Option<String>) -> Option<FetchMock> {
        if let Some(queue) = &mut self.fetches {
            let next_index = queue.iter().position(|x| same_endpoints(&x.url, url));
            let fetch = match next_index {
                Some(index) => queue.remove(index),
                None => FetchMock {
                    id: 0,
                    delay: None,
                    url: url.to_string(),
                    result: Err(crate::client::Error::http_request_send_error(
                        "Test fetch queue is empty",
                    )),
                },
            };
            let mut log = "Fetch".to_string();
            if fetch.id != 0 {
                log.push_str(&format!(" {}", fetch.id));
            }
            if let Some(delay) = fetch.delay {
                log.push_str(&format!(" {} ms ", delay));
            }
            log.push_str(" ");
            log.push_str(url);
            if let Some(body) = &body {
                log.push_str(&format!("\n  ⤷ {}", body));
            }
            println!("{}", log);
            Some(fetch)
        } else {
            None
        }
    }
}
