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

use super::websocket_link::WsConfig;
use crate::client::{ClientEnv, FetchMethod};
use crate::error::{AddNetworkUrl, ClientError, ClientResult};
use crate::net::gql::{GraphQLOperation, GraphQLOperationEvent, OrderBy, PostRequest};
use crate::net::server_info::ServerInfo;
use crate::net::websocket_link::WebsocketLink;
use crate::net::{Error, NetworkConfig};
use futures::{Future, Stream, StreamExt};
use serde_json::Value;
use tokio::sync::watch;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

pub const MAX_TIMEOUT: u32 = std::i32::MAX as u32;

pub(crate) struct Subscription {
    pub unsubscribe: Pin<Box<dyn Future<Output = ()> + Send>>,
    pub data_stream: Pin<Box<dyn Stream<Item = ClientResult<Value>> + Send>>,
}

pub(crate) struct ServerLink {
    config: NetworkConfig,
    endpoints: tokio::sync::RwLock<Vec<String>>,
    client_env: Arc<ClientEnv>,
    suspended: (watch::Sender<bool>, watch::Receiver<bool>),
    server_info: tokio::sync::RwLock<Option<ServerInfo>>,
    websocket_link: WebsocketLink,
    time_checked: AtomicBool,
}

impl ServerLink {
    pub fn new(config: NetworkConfig, client_env: Arc<ClientEnv>) -> ClientResult<Self> {
        let endpoints = config
            .endpoints
            .clone()
            .or(config.server_address.clone().map(|address| vec![address]))
            .ok_or(crate::client::Error::net_module_not_init())?;
        if endpoints.len() == 0 {
            return Err(crate::client::Error::net_module_not_init());
        }

        Ok(ServerLink {
            config: config.clone(),
            endpoints: tokio::sync::RwLock::new(endpoints),
            client_env: client_env.clone(),
            suspended: watch::channel(false),
            server_info: tokio::sync::RwLock::new(None),
            websocket_link: WebsocketLink::new(client_env.clone()),
            time_checked: AtomicBool::new(false),
        })
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

    async fn check_sync(&self) -> ClientResult<()> {
        if self.time_checked.load(Ordering::Relaxed) {
            return Ok(());
        }

        let client_lock = self.server_info.read().await;
        let info = client_lock.as_ref().unwrap();
        if info.server_version.supports_time {
            self.check_time_delta(&info.query_url, &self.config)
                .await?;
        }

        self.time_checked.store(true, Ordering::Relaxed);

        Ok(())
    }

    async fn init(&self) -> ClientResult<ServerInfo> {
        let mut futures = vec![];
        for address in self.endpoints.read().await.iter() {
            let queries_server = ServerInfo::expand_address(&address);
            futures.push(Box::pin(async move {
                ServerInfo::fetch(self.client_env.clone(), &queries_server).await
            }));
        }

        let mut server_info = Err(crate::client::Error::net_module_not_init());
        while futures.len() != 0 {
            let (result, _, remain_futures) = futures::future::select_all(futures).await;
            futures = remain_futures;
            server_info = result;
            if server_info.is_ok() {
                break;
            }
        }
        let server_info = server_info?;
        Ok(server_info)
    }

    async fn ensure_info(&self) -> ClientResult<()> {
        // wait for resume
        let mut suspended = self.suspended.1.clone();
        while Some(true) == suspended.recv().await {}

        if self.server_info.read().await.is_some() {
            return Ok(());
        }

        let mut data = self.server_info.write().await;
        if data.is_some() {
            return Ok(());
        }

        let inited_data = self.init().await?;

        self.websocket_link
            .set_config(WsConfig {
                url: inited_data.subscription_url.clone(),
                access_key: self.config.access_key.clone(),
                reconnect_timeout: self.config.reconnect_timeout,
            })
            .await;

        *data = Some(inited_data);

        Ok(())
    }

    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    pub async fn config_servers(&self) -> Vec<String> {
        self.endpoints.read().await.clone()
    }

    pub async fn query_url(&self) -> Option<String> {
        self.server_info
            .read()
            .await
            .as_ref()
            .map(|info| info.query_url.clone())
    }

    // Returns Stream with updates database fields by provided filter
    pub async fn subscribe(
        &self,
        table: &str,
        filter: &Value,
        fields: &str,
    ) -> ClientResult<Subscription> {
        self.ensure_info().await?;

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

        let result = self.fetch_operation(address, query, None).await?;

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
    pub async fn send_message(&self, key: &[u8], value: &[u8]) -> ClientResult<Option<ClientError>> {
        let request = PostRequest {
            id: base64::encode(key),
            body: base64::encode(value),
        };

        self.ensure_info().await?;
        self.check_sync().await?;
        let client_lock = self.server_info.read().await;
        let address = &client_lock.as_ref().unwrap().query_url;

        let result = self
            .fetch_operation(address, GraphQLOperation::post_requests(&[request]), None)
            .await;

        // send message is always successful in order to process case when server received message
        // but client didn't receive response
        if let Err(err) = &result {
            log::warn!("Post message error: {}", err.message);
        }

        Ok(result.err())
    }

    pub async fn suspend(&self) {
        let _ = self.suspended.0.broadcast(true);
        self.websocket_link.suspend().await;
    }

    pub async fn resume(&self) {
        let _ = self.suspended.0.broadcast(false);
        self.websocket_link.resume().await;
    }

    pub async fn fetch_endpoints(&self) -> ClientResult<Vec<String>> {
        self.ensure_info().await?;
        let client_lock = self.server_info.read().await;

        if !client_lock
            .as_ref()
            .unwrap()
            .server_version
            .supports_endpoints
        {
            return Err(Error::not_suppported("endpoints"));
        }

        let result = self
            .query_by_url(
                &client_lock.as_ref().unwrap().query_url,
                "%7Binfo%7Bendpoints%7D%7D",
            )
            .await
            .add_network_url(&self)
            .await?;

        serde_json::from_value(result["data"]["info"]["endpoints"].clone()).map_err(|_| {
            Error::invalid_server_response(format!(
                "Can not parse endpoints from response: {}",
                result
            ))
        })
    }

    pub async fn set_endpoints(&self, endpoints: Vec<String>) {
        *self.endpoints.write().await = endpoints;
    }
}
