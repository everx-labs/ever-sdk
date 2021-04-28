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
use crate::net::endpoint::Endpoint;
use crate::net::ton_gql::GraphQLQuery;
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

pub const MAX_TIMEOUT: u32 = std::i32::MAX as u32;
pub const MIN_RESUME_TIMEOUT: u32 = 500;
pub const MAX_RESUME_TIMEOUT: u32 = 3000;

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

pub(crate) struct NetworkState {
    client_env: Arc<ClientEnv>,
    config: NetworkConfig,
    endpoint_addresses: RwLock<Vec<String>>,
    bad_delivery_addresses: RwLock<HashSet<String>>,
    suspended: watch::Receiver<bool>,
    suspend_regulation: Arc<Mutex<SuspendRegulation>>,
    resume_timeout: AtomicU32,
    query_endpoint: RwLock<Option<Arc<Endpoint>>>,
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
        Self {
            client_env,
            config,
            endpoint_addresses: RwLock::new(endpoint_addresses),
            bad_delivery_addresses: RwLock::new(HashSet::new()),
            suspended: receiver,
            suspend_regulation: Arc::new(Mutex::new(regulation)),
            resume_timeout: AtomicU32::new(0),
            query_endpoint: RwLock::new(None),
            time_checked: AtomicBool::new(false),
        }
    }

    async fn suspend(&self, sender: &watch::Sender<bool>) {
        if !*self.suspended.borrow() {
            let _ = sender.broadcast(true);
            *self.query_endpoint.write().await = None;
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

    pub async fn set_endpoint_addresses(&self, addresses: Vec<String>) {
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

    pub async fn update_stat(&self, addresses: &Vec<String>, stat: EndpointStat) {
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

    pub async fn update_query_endpoint_latency(&self, latency: u64) {
        let mut endpoint_guard = self.query_endpoint.write().await;
        if let Some(old) = endpoint_guard.as_ref() {
            let mut endpoint = old.as_ref().clone();
            endpoint.latency = latency;
            endpoint.next_latency_detection_time =
                self.client_env.now_ms() + self.config.latency_detection_frequency as u64;
            *endpoint_guard = Some(Arc::new(endpoint));
        }
    }

    pub async fn config_servers(&self) -> Vec<String> {
        self.endpoint_addresses.read().await.clone()
    }

    pub async fn query_url(&self) -> Option<String> {
        self.query_endpoint
            .read()
            .await
            .as_ref()
            .map(|endpoint| endpoint.query_url.clone())
    }

    async fn check_time_delta(
        &self,
        endpoint: &Endpoint,
        out_of_sync_threshold: u32,
    ) -> ClientResult<()> {
        if endpoint.server_time_delta.abs() as u32 >= out_of_sync_threshold {
            Err(Error::clock_out_of_sync(
                endpoint.server_time_delta,
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

        let endpoint = self.get_query_endpoint().await?;
        self.check_time_delta(&endpoint, self.config.out_of_sync_threshold)
            .await?;

        self.time_checked.store(true, Ordering::Relaxed);

        Ok(())
    }

    async fn select_querying_endpoint(&self) -> ClientResult<Endpoint> {
        let is_better = |a: &ClientResult<Endpoint>, b: &ClientResult<Endpoint>| match (a, b) {
            (Ok(a), Ok(b)) => a.latency < b.latency,
            (Ok(_), Err(_)) => true,
            (Err(_), Err(_)) => true,
            _ => false,
        };
        let mut retry_count = 0i8;
        loop {
            let mut futures = vec![];
            for address in self.endpoint_addresses.read().await.iter() {
                let address = address.clone();
                futures.push(Box::pin(async move {
                    Endpoint::resolve(self.client_env.clone(), &self.config, &address).await
                }));
            }
            let mut selected = Err(crate::client::Error::net_module_not_init());
            while futures.len() != 0 {
                let (result, _, remain_futures) = futures::future::select_all(futures).await;
                if let Ok(endpoint) = &result {
                    if endpoint.latency <= self.config.max_latency as u64 {
                        return result;
                    }
                }
                futures = remain_futures;
                if is_better(&result, &selected) {
                    selected = result;
                }
            }
            if selected.is_ok() {
                return selected;
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

    pub async fn get_query_endpoint(&self) -> ClientResult<Arc<Endpoint>> {
        // wait for resume
        let mut suspended = self.suspended.clone();
        while Some(true) == suspended.recv().await {}

        if let Some(endpoint) = &*self.query_endpoint.read().await {
            return Ok(endpoint.clone());
        }

        let mut locked_query_endpoint = self.query_endpoint.write().await;
        if let Some(endpoint) = &*locked_query_endpoint {
            return Ok(endpoint.clone());
        }

        let fastest = Arc::new(self.select_querying_endpoint().await?);
        *locked_query_endpoint = Some(fastest.clone());
        Ok(fastest)
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
        let endpoint_addresses = config
            .endpoints
            .clone()
            .or(config.server_address.clone().map(|address| vec![address]))
            .ok_or(crate::client::Error::net_module_not_init())?;
        if endpoint_addresses.len() == 0 {
            return Err(crate::client::Error::net_module_not_init());
        }

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
            .start_operation(GraphQLQuery::with_subscription(table, filter, fields))
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

    pub(crate) async fn query(
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
        for (name, value) in Endpoint::http_headers() {
            headers.insert(name, value);
        }

        let network_retries_count = self.config.network_retries_count;
        let mut retry_count = 0;
        'retries: loop {
            let endpoint = if let Some(endpoint) = endpoint {
                Arc::new(endpoint.clone())
            } else {
                self.state.get_query_endpoint().await?
            };

            let result = self
                .client_env
                .fetch(
                    &endpoint.query_url,
                    FetchMethod::Post,
                    Some(headers.clone()),
                    Some(request.clone()),
                    query.timeout,
                )
                .await;

            if let Err(err) = &result {
                if crate::client::Error::is_network_error(err) {
                    self.state.internal_suspend().await;
                    self.websocket_link.suspend().await;
                    self.websocket_link.resume().await;
                    retry_count += 1;
                    if retry_count <= network_retries_count {
                        continue 'retries;
                    }
                }
            }
            let response = result?.body_as_json()?;

            return if let Some(error) = Self::try_extract_error(&response) {
                Err(error)
            } else {
                Ok(response)
            };
        }
    }

    pub async fn batch_query(
        &self,
        params: &[ParamsOfQueryOperation],
        endpoint: Option<Endpoint>,
    ) -> ClientResult<Vec<Value>> {
        let latency_detection_required = if endpoint.is_none() {
            let endpoint = self.state.get_query_endpoint().await?;
            self.client_env.now_ms() > endpoint.next_latency_detection_time
        } else {
            false
        };
        let mut query = GraphQLQuery::build(
            params,
            latency_detection_required,
            self.config.wait_for_timeout,
        );
        let mut result = self.query(&query, endpoint.as_ref()).await?;
        if latency_detection_required {
            let latency = query.get_latency(params, &result)?;
            if latency > self.config.max_latency as u64 {
                self.invalidate_querying_endpoint().await;
                query = GraphQLQuery::build(params, false, self.config.wait_for_timeout);
                result = self.query(&query, endpoint.as_ref()).await?;
            } else {
                self.state.update_query_endpoint_latency(latency).await;
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
        endpoint: Option<Endpoint>,
    ) -> ClientResult<Option<ClientError>> {
        let request = PostRequest {
            id: base64::encode(key),
            body: base64::encode(value),
        };

        self.state.check_sync().await?;

        let result = self
            .query(
                &GraphQLQuery::with_post_requests(&[request]),
                endpoint.as_ref(),
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

    pub async fn fetch_endpoint_addresses(&self) -> ClientResult<Vec<String>> {
        let endpoint = self.state.get_query_endpoint().await?;

        let result = query_by_url(
            &self.client_env,
            &endpoint.query_url,
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
        self.state.set_endpoint_addresses(endpoints).await;
    }

    pub async fn get_addresses_for_sending(&self) -> Vec<String> {
        self.state.get_addresses_for_sending().await
    }

    #[cfg(test)]
    pub async fn get_query_endpoint(&self) -> ClientResult<Arc<Endpoint>> {
        self.state.get_query_endpoint().await
    }

    pub async fn update_stat(&self, addresses: &Vec<String>, stat: EndpointStat) {
        self.state.update_stat(addresses, stat).await
    }

    pub async fn invalidate_querying_endpoint(&self) {
        self.state.invalidate_querying_endpoint().await
    }
}
