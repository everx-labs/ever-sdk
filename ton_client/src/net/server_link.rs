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

use crate::client::{ClientEnv, FetchMethod};
use crate::error::{AddNetworkUrl, ClientError, ClientResult};
use crate::net::endpoint::Endpoint;
use crate::net::ton_gql::GraphQLQuery;
use crate::net::types::NetworkQueriesProtocol;
use crate::net::websocket_link::WebsocketLink;
use crate::net::{
    Error, GraphQLQueryEvent, NetworkConfig, ParamsOfAggregateCollection, ParamsOfQueryCollection,
    ParamsOfQueryCounterparties, ParamsOfQueryOperation, ParamsOfWaitForCollection, PostRequest,
};
use futures::{Future, Stream, StreamExt};
use rand::seq::SliceRandom;
use serde_json::Value;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::{watch, Mutex, RwLock};

pub const MAX_TIMEOUT: u32 = i32::MAX as u32;
pub const MIN_RESUME_TIMEOUT: u32 = 500;
pub const MAX_RESUME_TIMEOUT: u32 = 3000;
pub const ENDPOINT_CACHE_TIMEOUT: u64 = 10 * 60 * 1000;

pub(crate) struct Subscription {
    pub unsubscribe: Pin<Box<dyn Future<Output = ()> + Send>>,
    pub data_stream: Pin<Box<dyn Stream<Item = ClientResult<Value>> + Send>>,
}

struct SuspendRegulation {
    sender: watch::Sender<bool>,
    internal_suspend: bool,
    external_suspend: bool,
}

pub(crate) enum EndpointStat {
    MessageDelivered,
    MessageUndelivered,
}

pub(crate) struct ResolvedEndpoint {
    pub endpoint: Arc<Endpoint>,
    pub time_added: u64,
}

pub(crate) struct NetworkState {
    client_env: Arc<ClientEnv>,
    config: NetworkConfig,
    endpoint_addresses: RwLock<Vec<String>>,
    has_multiple_endpoints: AtomicBool,
    bad_delivery_addresses: RwLock<HashSet<String>>,
    suspended: watch::Receiver<bool>,
    suspend_regulation: Arc<Mutex<SuspendRegulation>>,
    resume_timeout: AtomicU32,
    query_endpoint: RwLock<Option<Arc<Endpoint>>>,
    resolved_endpoints: RwLock<HashMap<String, ResolvedEndpoint>>,
}

async fn query_by_url(
    client_env: &ClientEnv,
    address: &str,
    query: &str,
    timeout: u32,
) -> ClientResult<Value> {
    let response = client_env
        .fetch(
            &format!("{}?query={}", address, query),
            FetchMethod::Get,
            None,
            None,
            timeout,
        )
        .await?;

    response.body_as_json()
}

impl NetworkState {
    pub fn new(
        client_env: Arc<ClientEnv>,
        config: NetworkConfig,
        endpoint_addresses: Vec<String>,
    ) -> Self {
        let (sender, receiver) = watch::channel(false);
        let regulation = SuspendRegulation {
            sender,
            internal_suspend: false,
            external_suspend: false,
        };
        let has_multiple_endpoints = AtomicBool::new(endpoint_addresses.len() > 1);
        Self {
            client_env,
            config,
            endpoint_addresses: RwLock::new(endpoint_addresses),
            has_multiple_endpoints,
            bad_delivery_addresses: RwLock::new(HashSet::new()),
            suspended: receiver,
            suspend_regulation: Arc::new(Mutex::new(regulation)),
            resume_timeout: AtomicU32::new(0),
            query_endpoint: RwLock::new(None),
            resolved_endpoints: Default::default(),
        }
    }

    pub fn has_multiple_endpoints(&self) -> bool {
        self.has_multiple_endpoints.load(Ordering::Relaxed)
    }

    async fn suspend(&self, sender: &watch::Sender<bool>) {
        if !*self.suspended.borrow() {
            let _ = sender.send(true);
            *self.query_endpoint.write().await = None;
        }
    }

    async fn resume(sender: &watch::Sender<bool>) {
        let _ = sender.send(false);
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

    pub async fn set_endpoint_addresses(&self, addresses: Vec<String>) {
        self.has_multiple_endpoints
            .store(addresses.len() > 1, Ordering::Relaxed);
        *self.endpoint_addresses.write().await = addresses;
    }

    pub async fn get_addresses_for_sending(&self) -> Vec<String> {
        let mut addresses = self.endpoint_addresses.read().await.clone();
        addresses.shuffle(&mut rand::thread_rng());
        let bad_delivery = self.bad_delivery_addresses.read().await.clone();
        if !bad_delivery.is_empty() {
            let mut i = 0;
            let mut processed = 0;
            while processed < addresses.len() {
                if bad_delivery.contains(&addresses[i]) {
                    let address = addresses.remove(i);
                    addresses.push(address);
                } else {
                    i += 1;
                }
                processed += 1;
            }
        }
        addresses
    }

    pub async fn update_stat(&self, addresses: &[String], stat: EndpointStat) {
        let bad_delivery = self.bad_delivery_addresses.read().await.clone();
        let addresses: HashSet<_> = addresses.iter().cloned().collect();
        let new_bad_delivery = match stat {
            EndpointStat::MessageDelivered => &bad_delivery - &addresses,
            EndpointStat::MessageUndelivered => &bad_delivery | &addresses,
        };
        if new_bad_delivery != bad_delivery {
            *self.bad_delivery_addresses.write().await = new_bad_delivery;
        }
    }

    pub async fn invalidate_querying_endpoint(&self) {
        *self.query_endpoint.write().await = None
    }

    pub async fn refresh_query_endpoint(&self) -> ClientResult<()> {
        let endpoint_guard = self.query_endpoint.write().await;
        if let Some(endpoint) = endpoint_guard.as_ref() {
            endpoint.refresh(&self.client_env, &self.config).await
        } else {
            Ok(())
        }
    }

    pub async fn config_servers(&self) -> Vec<String> {
        self.endpoint_addresses.read().await.clone()
    }

    pub async fn query_endpoint(&self) -> Option<Arc<Endpoint>> {
        self.query_endpoint.read().await.clone()
    }

    async fn check_sync_endpoint(&self, endpoint: &Endpoint) -> ClientResult<()> {
        let server_time_delta = endpoint.time_delta().abs();
        let threshold = self.config.out_of_sync_threshold;
        if server_time_delta >= threshold as i64 {
            Err(Error::clock_out_of_sync(server_time_delta, threshold))
        } else {
            Ok(())
        }
    }

    async fn check_sync(self: &Arc<NetworkState>, endpoint: Option<&Endpoint>) -> ClientResult<()> {
        if let Some(endpoint) = endpoint {
            self.check_sync_endpoint(endpoint).await
        } else {
            self.check_sync_endpoint(self.get_query_endpoint().await?.as_ref())
                .await
        }
    }

    pub async fn resolve_endpoint(&self, address: &str) -> ClientResult<Arc<Endpoint>> {
        if let Some(endpoint) = self.get_resolved_endpoint(address).await {
            Ok(endpoint)
        } else {
            let endpoint = Endpoint::resolve(&self.client_env, &self.config, address).await?;
            let endpoint = Arc::new(endpoint);
            self.add_resolved_endpoint(address.to_owned(), endpoint.clone()).await;
            Ok(endpoint)
        }
    }

    async fn select_querying_endpoint(self: &Arc<NetworkState>) -> ClientResult<Arc<Endpoint>> {
        let is_better = |a: &ClientResult<Arc<Endpoint>>, b: &ClientResult<Arc<Endpoint>>| match (a, b) {
            (Ok(a), Ok(b)) => a.latency() < b.latency(),
            (Ok(_), Err(_)) => true,
            (Err(_), Err(_)) => true,
            _ => false,
        };
        let mut retry_count = 0i8;
        loop {
            let mut futures = vec![];
            for address in self.endpoint_addresses.read().await.iter() {
                let address = address.clone();
                let self_copy = self.clone();
                futures.push(Box::pin(async move { self_copy.resolve_endpoint(&address).await }));
            }
            let mut selected = Err(crate::client::Error::net_module_not_init());
            let mut unauthorised = None;
            while futures.len() != 0 {
                let (result, _, remain_futures) = futures::future::select_all(futures).await;
                if let Ok(endpoint) = &result {
                    if endpoint.latency() <= self.config.max_latency as u64 {
                        if remain_futures.len() > 0 {
                            self.client_env.spawn(async move {
                                futures::future::join_all(remain_futures).await;
                            });
                        }
                        return result;
                    }
                }
                futures = remain_futures;
                if let Err(err) = &result {
                    if err.is_unauthorized() {
                        unauthorised = Some(err.clone());
                    }
                }
                if is_better(&result, &selected) {
                    selected = result;
                }
            }
            if selected.is_ok() {
                return selected;
            }
            if let Some(unauthorised) = unauthorised {
                return Err(unauthorised);
            }
            retry_count += 1;
            if retry_count > self.config.network_retries_count {
                return selected;
            }
            if retry_count > 1 {
                let delay = (100 * (retry_count - 1) as u64).max(5000);
                let _ = self.client_env.set_timer(delay).await;
            }
        }
    }

    pub async fn get_query_endpoint(self: &Arc<NetworkState>) -> ClientResult<Arc<Endpoint>> {
        // wait for resume
        let mut suspended = self.suspended.clone();
        while *suspended.borrow() {
            let _ = suspended.changed().await;
        }

        if let Some(endpoint) = &*self.query_endpoint.read().await {
            return Ok(endpoint.clone());
        }

        let mut locked_query_endpoint = self.query_endpoint.write().await;
        if let Some(endpoint) = &*locked_query_endpoint {
            return Ok(endpoint.clone());
        }
        let fastest = self.select_querying_endpoint().await?;
        *locked_query_endpoint = Some(fastest.clone());
        Ok(fastest)
    }

    pub async fn get_all_endpoint_addresses(&self) -> ClientResult<Vec<String>> {
        Ok(self.endpoint_addresses.read().await.clone())
    }

    pub async fn add_resolved_endpoint(&self, address: String, endpoint: Arc<Endpoint>) {
        let mut lock = self.resolved_endpoints.write().await;
        lock.insert(address, ResolvedEndpoint { endpoint, time_added: self.client_env.now_ms() });
    }

    pub async fn get_resolved_endpoint(&self, address: &str) -> Option<Arc<Endpoint>> {
        let lock = self.resolved_endpoints.read().await;
        lock
            .get(address)
            .and_then(|endpoint| 
                if endpoint.time_added + ENDPOINT_CACHE_TIMEOUT > self.client_env.now_ms() {
                    Some(endpoint.endpoint.clone())
                } else {
                    None
                }
            )
    }
}

pub(crate) struct ServerLink {
    config: NetworkConfig,
    pub(crate) client_env: Arc<ClientEnv>,
    websocket_link: WebsocketLink,
    state: Arc<NetworkState>,
}

fn strip_endpoint(endpoint: &str) -> &str {
    endpoint
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches("/")
        .trim_end_matches("\\")
}

fn same_endpoint(a: &str, b: &str) -> bool {
    strip_endpoint(a) == strip_endpoint(b)
}

fn replace_endpoints(endpoints: Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    for endpoint in endpoints {
        if !result.iter().any(|val| same_endpoint(val, &endpoint)) {
            result.push(endpoint);
        }
    }

    result
}

impl ServerLink {
    pub fn new(config: NetworkConfig, client_env: Arc<ClientEnv>) -> ClientResult<Self> {
        let endpoint_addresses = config
            .endpoints
            .clone()
            .or(config.server_address.clone().map(|address| vec![address]))
            .ok_or(crate::client::Error::net_module_not_init())?;
        if endpoint_addresses.len() == 0 {
            return Err(crate::client::Error::net_module_not_init());
        }
        let endpoint_addresses = replace_endpoints(endpoint_addresses);

        let state = Arc::new(NetworkState::new(
            client_env.clone(),
            config.clone(),
            endpoint_addresses,
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

    pub fn state(&self) -> Arc<NetworkState> {
        self.state.clone()
    }

    // Returns Stream with updates database fields by provided filter
    pub async fn subscribe_collection(
        &self,
        table: &str,
        filter: &Value,
        fields: &str,
    ) -> ClientResult<Subscription> {
        let event_receiver = self
            .websocket_link
            .start_operation(GraphQLQuery::with_collection_subscription(
                table, filter, fields,
            ))
            .await?;
        let event_receiver = tokio_stream::wrappers::ReceiverStream::new(event_receiver);

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
                    GraphQLQueryEvent::Id(id) => {
                        *operation_id.lock().await = id;
                        None
                    }
                    GraphQLQueryEvent::Data(value) => Some(Ok(value[&collection_name].clone())),
                    GraphQLQueryEvent::Error(error) => Some(Err(error)),
                    GraphQLQueryEvent::Complete => Some(Ok(Value::Null)),
                }
            }
        });
        Ok(Subscription {
            data_stream: Box::pin(data_receiver),
            unsubscribe: Box::pin(unsubscribe),
        })
    }

    // Returns Stream with updates database fields by provided filter
    pub async fn subscribe(
        &self,
        subscription: String,
        variables: Option<Value>,
    ) -> ClientResult<Subscription> {
        let event_receiver = self
            .websocket_link
            .start_operation(GraphQLQuery::with_subscription(subscription, variables))
            .await?;
        let event_receiver = tokio_stream::wrappers::ReceiverStream::new(event_receiver);

        let operation_id = Arc::new(Mutex::new(0u32));
        let unsubscribe_operation_id = operation_id.clone();

        let link = self.websocket_link.clone();
        let unsubscribe = async move {
            let id = *unsubscribe_operation_id.lock().await;
            link.stop_operation(id).await;
        };

        let data_receiver = event_receiver.filter_map(move |event| {
            let operation_id = operation_id.clone();
            async move {
                match event {
                    GraphQLQueryEvent::Id(id) => {
                        *operation_id.lock().await = id;
                        None
                    }
                    GraphQLQueryEvent::Data(value) => Some(Ok(value.clone())),
                    GraphQLQueryEvent::Error(error) => Some(Err(error)),
                    GraphQLQueryEvent::Complete => Some(Ok(Value::Null)),
                }
            }
        });
        Ok(Subscription {
            data_stream: Box::pin(data_receiver),
            unsubscribe: Box::pin(unsubscribe),
        })
    }

    pub(crate) async fn query_http(
        &self,
        query: &GraphQLQuery,
        endpoint: Option<&Endpoint>,
    ) -> ClientResult<Value> {
        let request = json!({
            "query": query.query,
            "variables": query.variables,
        })
        .to_string();

        let mut headers = HashMap::new();
        headers.insert("content-type".to_owned(), "application/json".to_owned());
        for (name, value) in Endpoint::http_headers(&self.config) {
            headers.insert(name, value);
        }

        let network_retries_count = self.config.network_retries_count;
        let mut current_endpoint: Option<Arc<Endpoint>>;
        let mut retry_count = 0;
        'retries: loop {
            let endpoint = if let Some(endpoint) = endpoint {
                endpoint
            } else {
                current_endpoint = Some(self.state.get_query_endpoint().await?.clone());
                current_endpoint.as_ref().unwrap()
            };
            let result = self
                .client_env
                .fetch(
                    &endpoint.query_url,
                    FetchMethod::Post,
                    Some(headers.clone()),
                    Some(request.clone()),
                    query.timeout.unwrap_or(self.config.query_timeout),
                )
                .await;

            let result = match result {
                Err(err) => Err(err),
                Ok(response) => {
                    if response.status == 401 {
                        Err(Error::unauthorized(&response))
                    } else {
                        match response.body_as_json() {
                            Err(err) => Err(err),
                            Ok(value) => match Error::try_extract_graphql_error(&value) {
                                Some(err) => Err(err),
                                None => Ok(value),
                            },
                        }
                    }
                }
            };

            if let Err(err) = &result {
                if crate::client::Error::is_network_error(err) {
                    let endpoint_count = self
                        .state
                        .get_all_endpoint_addresses()
                        .await
                        .map(|x| x.len())
                        .unwrap_or(0);
                    if endpoint_count > 1 {
                        self.state.internal_suspend().await;
                        self.websocket_link.suspend().await;
                        self.websocket_link.resume().await;
                    }
                    retry_count += 1;
                    if retry_count <= network_retries_count {
                        continue 'retries;
                    }
                }
            }

            return result;
        }
    }

    pub(crate) async fn query_ws(&self, query: &GraphQLQuery) -> ClientResult<Value> {
        let mut receiver = self.websocket_link.start_operation(query.clone()).await?;
        let mut id = None::<u32>;
        let mut result = Ok(Value::Null);
        loop {
            match receiver.recv().await {
                Some(GraphQLQueryEvent::Id(received_id)) => id = Some(received_id),
                Some(GraphQLQueryEvent::Data(data)) => {
                    result = Ok(json!({ "data": data }));
                    break;
                }
                Some(GraphQLQueryEvent::Complete) => break,
                Some(GraphQLQueryEvent::Error(err)) => {
                    result = Err(err);
                    break;
                }
                None => break,
            }
        }
        if let Some(id) = id {
            self.websocket_link.stop_operation(id).await;
        }
        result
    }

    pub(crate) async fn query(
        &self,
        query: &GraphQLQuery,
        endpoint: Option<&Endpoint>,
    ) -> ClientResult<Value> {
        match self.config.queries_protocol {
            NetworkQueriesProtocol::HTTP => self.query_http(query, endpoint).await,
            NetworkQueriesProtocol::WS => self.query_ws(query).await,
        }
    }

    pub async fn batch_query(
        &self,
        params: &[ParamsOfQueryOperation],
        endpoint: Option<Endpoint>,
    ) -> ClientResult<Vec<Value>> {
        let latency_detection_required = if endpoint.is_some() {
            false
        } else if self.state.has_multiple_endpoints() {
            let endpoint = self.state.get_query_endpoint().await?;
            self.client_env.now_ms() > endpoint.next_latency_detection_time()
        } else {
            false
        };
        let mut query = GraphQLQuery::build(
            params,
            latency_detection_required,
            self.config.wait_for_timeout,
        );
        let info_request_time = self.client_env.now_ms();
        let mut result = self.query(&query, endpoint.as_ref()).await?;
        if latency_detection_required {
            let current_endpoint = self.state.get_query_endpoint().await?;
            let server_info = query.get_server_info(&params, &result)?;
            current_endpoint.apply_server_info(
                &self.client_env,
                &self.config,
                info_request_time,
                &server_info,
            )?;
            if current_endpoint.latency() > self.config.max_latency as u64 {
                self.invalidate_querying_endpoint().await;
                query = GraphQLQuery::build(params, false, self.config.wait_for_timeout);
                result = self.query(&query, endpoint.as_ref()).await?;
            }
        }
        query.get_results(params, &result)
    }

    pub async fn query_collection(
        &self,
        params: ParamsOfQueryCollection,
        endpoint: Option<Endpoint>,
    ) -> ClientResult<Value> {
        Ok(self
            .batch_query(&[ParamsOfQueryOperation::QueryCollection(params)], endpoint)
            .await?
            .remove(0))
    }

    pub async fn wait_for_collection(
        &self,
        params: ParamsOfWaitForCollection,
        endpoint: Option<Endpoint>,
    ) -> ClientResult<Value> {
        Ok(self
            .batch_query(
                &[ParamsOfQueryOperation::WaitForCollection(params)],
                endpoint,
            )
            .await?
            .remove(0))
    }

    pub async fn aggregate_collection(
        &self,
        params: ParamsOfAggregateCollection,
        endpoint: Option<Endpoint>,
    ) -> ClientResult<Value> {
        Ok(self
            .batch_query(
                &[ParamsOfQueryOperation::AggregateCollection(params)],
                endpoint,
            )
            .await?
            .remove(0))
    }

    pub async fn query_counterparties(
        &self,
        params: ParamsOfQueryCounterparties,
    ) -> ClientResult<Value> {
        Ok(self
            .batch_query(&[ParamsOfQueryOperation::QueryCounterparties(params)], None)
            .await?
            .remove(0))
    }

    // Sends message to node
    pub async fn send_message(
        &self,
        key: &[u8],
        value: &[u8],
        endpoint: Option<&Endpoint>,
    ) -> ClientResult<Option<ClientError>> {
        let request = PostRequest {
            id: base64::encode(key),
            body: base64::encode(value),
        };

        self.state.check_sync(endpoint).await?;

        let result = self
            .query(&GraphQLQuery::with_post_requests(&[request]), endpoint)
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

    pub async fn fetch_endpoint_addresses(&self) -> ClientResult<Vec<String>> {
        let endpoint = self.state.get_query_endpoint().await?;

        let result = query_by_url(
            &self.client_env,
            &endpoint.query_url,
            "%7Binfo%7Bendpoints%7D%7D",
            self.config.query_timeout,
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
        self.state.set_endpoint_addresses(endpoints).await;
    }

    pub async fn get_addresses_for_sending(&self) -> Vec<String> {
        self.state.get_addresses_for_sending().await
    }

    pub async fn get_query_endpoint(&self) -> ClientResult<Arc<Endpoint>> {
        self.state.get_query_endpoint().await
    }

    pub async fn get_all_endpoint_addresses(&self) -> ClientResult<Vec<String>> {
        self.state.get_all_endpoint_addresses().await
    }

    pub async fn update_stat(&self, addresses: &[String], stat: EndpointStat) {
        self.state.update_stat(addresses, stat).await
    }

    pub async fn invalidate_querying_endpoint(&self) {
        self.state.invalidate_querying_endpoint().await
    }
}
