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
use crate::error::{AddNetworkUrl, ClientError, ClientResult};
use crate::net::server_info::ServerInfo;
use crate::net::ton_gql::GraphQLOperation;
use crate::net::websocket_link::WebsocketLink;
use crate::net::{
    Error, GraphQLOperationEvent, NetworkConfig, ParamsOfAggregateCollection,
    ParamsOfQueryCollection, ParamsOfQueryOperation, ParamsOfWaitForCollection, PostRequest,
};
use futures::{Future, Stream, StreamExt};
use serde_json::Value;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::sync::{Mutex, RwLock, watch};

pub const MAX_TIMEOUT: u32 = std::i32::MAX as u32;
pub const MIN_RESUME_TIMEOUT: u32 = 500;
pub const MAX_RESUME_TIMEOUT: u32 = 3000;
pub const FETCH_ADDITIONAL_TIMEOUT: u32 = 5000;

pub(crate) struct Subscription {
    pub unsubscribe: Pin<Box<dyn Future<Output = ()> + Send>>,
    pub data_stream: Pin<Box<dyn Stream<Item = ClientResult<Value>> + Send>>,
}

struct SuspendRegulation {
    sender: watch::Sender<bool>,
    internal_suspend: bool,
    external_suspend: bool,
}

pub(crate) struct NetworkState {
    client_env: Arc<ClientEnv>,
    endpoints: RwLock<Vec<String>>,
    suspended: watch::Receiver<bool>,
    suspend_regulation: Arc<Mutex<SuspendRegulation>>,
    resume_timeout: AtomicU32,
    server_info: RwLock<Option<Arc<ServerInfo>>>,
    out_of_sync_threshold: u32,
    time_checked: AtomicBool,
}

async fn query_by_url(client_env: &ClientEnv, address: &str, query: &str) -> ClientResult<Value> {
    let response = client_env
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

impl NetworkState {
    pub fn new(client_env: Arc<ClientEnv>, endpoints: Vec<String>, out_of_sync_threshold: u32) -> Self {
        let (sender, receiver) = watch::channel(false);
        let regulation = SuspendRegulation {
            sender,
            internal_suspend: false,
            external_suspend: false,
        };
        Self {
            client_env,
            endpoints: RwLock::new(endpoints),
            suspended: receiver,
            suspend_regulation: Arc::new(Mutex::new(regulation)),
            resume_timeout: AtomicU32::new(0),
            server_info: RwLock::new(None),
            out_of_sync_threshold,
            time_checked: AtomicBool::new(false),
        }
    }

    async fn suspend(&self, sender: &watch::Sender<bool>) {
        if !*self.suspended.borrow() {
            let _ = sender.broadcast(true);
            *self.server_info.write().await = None;
        }
    }

    async fn resume(sender: &watch::Sender<bool>) {
        let _ = sender.broadcast(false);
    }

    pub async fn external_suspend(&self) {
        let mut regulation = self.suspend_regulation.lock().await;
        regulation.external_suspend = true;
        self.suspend(&regulation.sender).await;
    }

    pub async fn external_resume(&self) {
        let mut regulation = self.suspend_regulation.lock().await;
        regulation.external_suspend = false;
        if !regulation.internal_suspend {
            Self::resume(&regulation.sender).await;
        }
    }

    pub async fn internal_suspend(&self) {
        let mut regulation = self.suspend_regulation.lock().await;
        if regulation.internal_suspend {
            return;
        }

        regulation.internal_suspend = true;
        self.suspend(&regulation.sender).await;

        let timeout = self.resume_timeout.load(Ordering::Relaxed);
        let next_timeout = min(max(timeout * 2, MIN_RESUME_TIMEOUT), MAX_RESUME_TIMEOUT); // 0, 0.5, 1, 2, 3, 3, 3...
        self.resume_timeout.store(next_timeout, Ordering::Relaxed);
        log::debug!("Internal resume timeout {}", timeout);

        let env = self.client_env.clone();
        let regulation = self.suspend_regulation.clone();

        self.client_env.spawn(async move {
            let _ = env.set_timer(timeout as u64).await;
            let mut regulation = regulation.lock().await;
            regulation.internal_suspend = false;
            if !regulation.external_suspend {
                Self::resume(&regulation.sender).await;
            }
        });
    }

    pub async fn set_endpoints(&self, endpoints: Vec<String>) {
        *self.endpoints.write().await = endpoints;
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

    async fn get_time_delta(&self, address: &str) -> ClientResult<i64> {
        let start = self.client_env.now_ms() as i64;
        let response = query_by_url(&self.client_env, address, "%7Binfo%7Btime%7D%7D").await?;
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

    async fn check_time_delta(&self, address: &str, out_of_sync_threshold: u32) -> ClientResult<()> {
        let delta = self.get_time_delta(address).await?;
        if delta.abs() as u32 >= out_of_sync_threshold {
            Err(Error::clock_out_of_sync(
                delta,
                out_of_sync_threshold,
            ))
        } else {
            Ok(())
        }
    }

    async fn check_sync(&self) -> ClientResult<()> {
        if self.time_checked.load(Ordering::Relaxed) {
            return Ok(());
        }

        let info = self.get_info().await?;
        if info.server_version.supports_time {
            self.check_time_delta(&info.query_url, self.out_of_sync_threshold).await?;
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
        server_info
    }

    pub async fn get_info(&self) -> ClientResult<Arc<ServerInfo>> {
        // wait for resume
        let mut suspended = self.suspended.clone();
        while Some(true) == suspended.recv().await {}

        if let Some(info) = &*self.server_info.read().await {
            return Ok(info.clone());
        }

        let mut data = self.server_info.write().await;
        if let Some(info) = &*data {
            return Ok(info.clone());
        }

        let inited_data = Arc::new(self.init().await?);

        *data = Some(inited_data.clone());

        Ok(inited_data)
    }
}

pub(crate) struct ServerLink {
    config: NetworkConfig,
    client_env: Arc<ClientEnv>,
    websocket_link: WebsocketLink,
    state: Arc<NetworkState>,
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

        let state = Arc::new(NetworkState::new(
            client_env.clone(),
            endpoints,
            config.out_of_sync_threshold
        ));

        Ok(ServerLink {
            config: config.clone(),
            client_env: client_env.clone(),
            state: state.clone(),
            websocket_link: WebsocketLink::new(client_env, state, config),
        })
    }

    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    pub async fn config_servers(&self) -> Vec<String> {
        self.state.config_servers().await
    }

    pub async fn query_url(&self) -> Option<String> {
        self.state.query_url().await
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
            .start_operation(GraphQLOperation::with_subscription(table, filter, fields))
            .await?;

        let operation_id = Arc::new(Mutex::new(0u32));
        let unsubscribe_operation_id = operation_id.clone();

        let link = self.websocket_link.clone();
        let unsubscribe = async move {
            let id = *unsubscribe_operation_id.lock().await;
            link.stop_operation(id).await;
        };

        let collection_name = table.to_string();
        let data_receiver = event_receiver.filter_map(move |event| {
            let operation_id = operation_id.clone();
            let collection_name = collection_name.clone();
            async move {
                match event {
                    GraphQLOperationEvent::Id(id) => {
                        *operation_id.lock().await = id;
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
        operation: GraphQLOperation,
        timeout: Option<u32>,
    ) -> ClientResult<Value> {
        let info = self.state.get_info().await?;

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

        let result = self
            .client_env
            .fetch(
                &info.query_url,
                FetchMethod::Post,
                Some(headers),
                Some(request),
                timeout,
            )
            .await;

        if let Err(err) = &result {
            if crate::client::Error::is_network_error(err) {
                self.state.internal_suspend().await;
                self.websocket_link.suspend().await;
                self.websocket_link.resume().await;
            }
        }

        let response = result?.body_as_json()?;

        if let Some(error) = Self::try_extract_error(&response) {
            Err(error)
        } else {
            Ok(response)
        }
    }

    pub async fn batch_query(&self, params: &[ParamsOfQueryOperation]) -> ClientResult<Vec<Value>> {
        let op = GraphQLOperation::build(params, self.config.wait_for_timeout);
        let result = self.fetch_operation(op, None).await?;
        let data = &result["data"];
        let mut results = Vec::new();
        for i in 0..params.len() {
            let result_name = if params.len() > 1 {
                format!("q{}", i + 1)
            } else {
                params[0].query_name()
            };
            let mut result_data = &data[result_name.as_str()];
            if result_data.is_null() {
                return Err(Error::invalid_server_response(format!(
                    "Invalid query answer: {}",
                    result
                )));
            }
            if let ParamsOfQueryOperation::WaitForCollection(_) = params[i] {
                result_data = &result_data[0];
                if result_data.is_null() {
                    return Err(Error::wait_for_timeout());
                }
            }
            results.push(result_data.clone());
        }
        Ok(results)
    }

    pub async fn query_collection(&self, params: ParamsOfQueryCollection) -> ClientResult<Value> {
        Ok(self
            .batch_query(&[ParamsOfQueryOperation::QueryCollection(params)])
            .await?
            .remove(0))
    }

    pub async fn wait_for_collection(
        &self,
        params: ParamsOfWaitForCollection,
    ) -> ClientResult<Value> {
        Ok(self
            .batch_query(&[ParamsOfQueryOperation::WaitForCollection(params)])
            .await?
            .remove(0))
    }

    pub async fn aggregate_collection(
        &self,
        params: ParamsOfAggregateCollection,
    ) -> ClientResult<Value> {
        Ok(self
            .batch_query(&[ParamsOfQueryOperation::AggregateCollection(params)])
            .await?
            .remove(0))
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
        };
        Ok(self.fetch_operation(
            query,
            timeout.map(|value| value + FETCH_ADDITIONAL_TIMEOUT)
        ).await?)
    }

    // Sends message to node
    pub async fn send_message(
        &self,
        key: &[u8],
        value: &[u8],
    ) -> ClientResult<Option<ClientError>> {
        let request = PostRequest {
            id: base64::encode(key),
            body: base64::encode(value),
        };

        self.state.check_sync().await?;

        let result = self
            .fetch_operation(
                GraphQLOperation::with_post_requests(&[request]),
                None,
            )
            .await;

        // send message is always successful in order to process case when server received message
        // but client didn't receive response
        if let Err(err) = &result {
            log::warn!("Post message error: {}", err.message);
        }

        Ok(result.err())
    }

    pub async fn suspend(&self) {
        self.state.external_suspend().await;
        self.websocket_link.suspend().await;
    }

    pub async fn resume(&self) {
        self.state.external_resume().await;
        self.websocket_link.resume().await;
    }

    pub async fn fetch_endpoints(&self) -> ClientResult<Vec<String>> {
        let info = self.state.get_info().await?;

        if !info.server_version.supports_endpoints {
            return Err(Error::not_suppported("endpoints"));
        }

        let result = query_by_url(
                &self.client_env,
                &info.query_url,
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
        self.state.set_endpoints(endpoints).await;
    }
}
