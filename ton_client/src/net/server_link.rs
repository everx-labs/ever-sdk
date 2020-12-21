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

use crate::client::{ClientEnv, FetchMethod};
use crate::error::{ClientError, ClientResult};
use crate::net::gql::{GraphQLOperation, GraphQLOperationEvent, OrderBy, PostRequest};
use crate::net::server_info::ServerInfo;
use crate::net::websocket_link::WebsocketLink;
use crate::net::{Error, NetworkConfig};
use futures::{Future, Stream, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub const MAX_TIMEOUT: u32 = std::i32::MAX as u32;

pub(crate) struct Subscription {
    pub unsubscribe: Pin<Box<dyn Future<Output = ()> + Send>>,
    pub data_stream: Pin<Box<dyn Stream<Item = ClientResult<Value>> + Send>>,
}

pub(crate) struct ServerLink {
    config: NetworkConfig,
    client_env: Arc<ClientEnv>,
    suspended: AtomicBool,
    server_info: tokio::sync::RwLock<Option<ServerInfo>>,
    // TODO: use tokio::sync:RwLock when SDK core is fully async
    query_url: std::sync::RwLock<Option<String>>,
    websocket_link: WebsocketLink,
}

impl ServerLink {
    pub fn new(config: NetworkConfig, client_env: Arc<ClientEnv>) -> Self {
        ServerLink {
            config: config.clone(),
            client_env: client_env.clone(),
            suspended: AtomicBool::new(false),
            query_url: std::sync::RwLock::new(None),
            server_info: tokio::sync::RwLock::new(None),
            websocket_link: WebsocketLink::new(config, client_env.clone()),
        }
    }

    async fn query_by_url(&self, address: &str, query: &str) -> ClientResult<Value> {
        let response = self
            .client_env
            .fetch(
                &format!("{}?query={}", address, query),
                FetchMethod::Get,
                None,
                None,
                None,
            )
            .await?;

        response.body_as_json()
    }

    async fn get_time_delta(&self, address: &str) -> ClientResult<i64> {
        let start = self.client_env.now_ms() as i64;
        let response = self.query_by_url(address, "%7Binfo%7Btime%7D%7D").await?;
        let end = self.client_env.now_ms() as i64;
        let server_time =
            response["data"]["info"]["time"]
                .as_i64()
                .ok_or(Error::invalid_server_response(format!(
                    "No time in response: {}",
                    response
                )))?;

        Ok(server_time - (start + (end - start) / 2))
    }

    async fn check_time_delta(&self, address: &str, config: &NetworkConfig) -> ClientResult<()> {
        let delta = self.get_time_delta(address).await?;
        if delta.abs() as u32 >= config.out_of_sync_threshold {
            Err(Error::clock_out_of_sync(
                delta,
                config.out_of_sync_threshold,
            ))
        } else {
            Ok(())
        }
    }

    async fn init(&self, config: &NetworkConfig) -> ClientResult<ServerInfo> {
        let queries_server = ServerInfo::expand_address(&config.server_address);
        let server_info = ServerInfo::fetch(self.client_env.clone(), &queries_server).await?;

        if server_info.server_version.supports_time {
            self.check_time_delta(&queries_server, config).await?;
        }

        Ok(server_info)
    }

    async fn ensure_info(&self) -> ClientResult<()> {
        if self.suspended.load(Ordering::Relaxed) {
            return Err(Error::network_module_suspended());
        }

        if self.server_info.read().await.is_some() {
            return Ok(());
        }

        let mut data = self.server_info.write().await;
        if data.is_some() {
            return Ok(());
        }

        let inited_data = self.init(&self.config).await?;
        *self.query_url.write().unwrap() = Some(inited_data.query_url.clone());
        *data = Some(inited_data);

        Ok(())
    }

    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    pub fn config_server(&self) -> &str {
        &self.config.server_address
    }

    pub fn query_url(&self) -> Option<String> {
        self.query_url.read().unwrap().clone()
    }

    // Returns Stream with updates database fields by provided filter
    pub async fn subscribe(
        &self,
        table: &str,
        filter: &Value,
        fields: &str,
    ) -> ClientResult<Subscription> {
        let event_receiver = self
            .websocket_link
            .start_operation(GraphQLOperation::subscription(table, filter, fields))
            .await?;

        let operation_id = Arc::new(Mutex::new(0u32));
        let unsubscribe_operation_id = operation_id.clone();

        let link = self.websocket_link.clone();
        let unsubscribe = async move {
            let id = unsubscribe_operation_id.lock().ok().map(|g| *g);
            if let Some(id) = id {
                link.stop_operation(id).await;
            }
        };

        let collection_name = table.to_string();
        let data_receiver = event_receiver.filter_map(move |event| {
            let operation_id = operation_id.clone();
            let collection_name = collection_name.clone();
            async move {
                match event {
                    GraphQLOperationEvent::Id(id) => {
                        if let Ok(mut guard) = operation_id.lock() {
                            *guard = id;
                        }
                        None
                    }
                    GraphQLOperationEvent::Data(value) => Some(Ok(value[&collection_name].clone())),
                    GraphQLOperationEvent::Error(error) => Some(Err(error)),
                    GraphQLOperationEvent::Complete => Some(Ok(Value::Null)),
                }
            }
        });
        Ok(Subscription {
            data_stream: Box::pin(data_receiver),
            unsubscribe: Box::pin(unsubscribe),
        })
    }

    pub fn try_extract_error(value: &Value) -> Option<ClientError> {
        let errors = if let Some(payload) = value.get("payload") {
            payload.get("errors")
        } else {
            value.get("errors")
        };

        if let Some(errors) = errors {
            if let Some(errors) = errors.as_array() {
                if errors.len() > 0 {
                    if let Some(error) = errors.get(0) {
                        if let Some(message) = error.get("message") {
                            if let Some(string) = message.as_str() {
                                return Some(Error::graphql_error(string));
                            }
                        }
                    }
                }
            }
        }

        return None;
    }

    async fn fetch_operation(
        &self,
        address: &str,
        operation: GraphQLOperation,
        timeout: Option<u32>,
    ) -> ClientResult<Value> {
        let request = json!({
            "query": operation.query,
            "variables": operation.variables,
        })
        .to_string();

        let mut headers = HashMap::new();
        headers.insert("content-type".to_owned(), "application/json".to_owned());
        for (name, value) in ServerInfo::http_headers() {
            headers.insert(name, value);
        }

        let response = self
            .client_env
            .fetch(
                address,
                FetchMethod::Post,
                Some(headers),
                Some(request),
                timeout,
            )
            .await?;

        let response = response.body_as_json()?;

        if let Some(error) = Self::try_extract_error(&response) {
            Err(error)
        } else {
            Ok(response)
        }
    }

    // Returns Stream with GraphQL query answer
    pub async fn query_collection(
        &self,
        table: &str,
        filter: &Value,
        fields: &str,
        order_by: Option<Vec<OrderBy>>,
        limit: Option<u32>,
        timeout: Option<u32>,
    ) -> ClientResult<Value> {
        let query = GraphQLOperation::query(table, filter, fields, order_by, limit, timeout);

        self.ensure_info().await?;
        let client_lock = self.server_info.read().await;
        let address = &client_lock.as_ref().unwrap().query_url;

        let result = self.fetch_operation(address, query, timeout).await?;

        // try to extract the record value from the answer
        let records_array = &result["data"][&table];
        if records_array.is_null() {
            Err(Error::invalid_server_response(format!(
                "Invalid query answer: {}",
                result
            )))
        } else {
            Ok(records_array.clone())
        }
    }

    // Returns GraphQL query answer
    pub async fn query(
        &self,
        query: &str,
        variables: Option<Value>,
        timeout: Option<u32>,
    ) -> ClientResult<Value> {
        let query = GraphQLOperation {
            query: query.into(),
            variables,
            operation_name: None,
        };
        self.ensure_info().await?;
        let client_lock = self.server_info.read().await;
        let address = &client_lock.as_ref().unwrap().query_url;
        Ok(self.fetch_operation(address, query, timeout).await?)
    }

    // Executes GraphQL query, waits for result and returns recieved value
    pub async fn wait_for(
        &self,
        table: &str,
        filter: &Value,
        fields: &str,
        timeout: Option<u32>,
    ) -> ClientResult<Value> {
        let value = self
            .query_collection(
                table,
                filter,
                fields,
                None,
                None,
                timeout.or(Some(self.config.wait_for_timeout)),
            )
            .await?;

        if !value[0].is_null() {
            Ok(value[0].clone())
        } else {
            Err(Error::wait_for_timeout())
        }
    }

    // Sends message to node
    pub async fn send_message(&self, key: &[u8], value: &[u8]) -> ClientResult<()> {
        let request = PostRequest {
            id: base64::encode(key),
            body: base64::encode(value),
        };

        self.ensure_info().await?;
        let client_lock = self.server_info.read().await;
        let address = &client_lock.as_ref().unwrap().query_url;

        let result = self
            .fetch_operation(address, GraphQLOperation::post_requests(&[request]), None)
            .await;

        // send message is always successful in order to process case when server received message
        // but client didn't receive response
        if let Err(err) = result {
            log::warn!("Post message error: {}", err.message);
        }

        Ok(())
    }

    pub async fn suspend(&self) {
        self.suspended.store(true, Ordering::Relaxed);
        self.websocket_link.suspend().await;
    }

    pub async fn resume(&self) {
        self.suspended.store(false, Ordering::Relaxed);
        self.websocket_link.resume().await;
    }
}
