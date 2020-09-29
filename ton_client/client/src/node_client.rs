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
use crate::net::Error;
use crate::error::{ApiResult, ApiError};
use futures::{Future, SinkExt, Stream, StreamExt};
use serde_json::Value;
use rand::RngCore;
use std::collections::HashMap;
use std::sync::Arc;
use std::pin::Pin;
use std::iter::FromIterator;

pub const MAX_TIMEOUT: u32 = std::i32::MAX as u32;

pub const DEFAULT_RETRIES_COUNT: u8 = 5;
pub const DEFAULT_EXPIRATION_TIMEOUT: u32 = 40000;
pub const DEFAULT_PROCESSING_TIMEOUT: u32 = 40000;
pub const DEFAULT_TIMEOUT_GROW_FACTOR: f32 = 1.5;
pub const DEFAULT_WAIT_TIMEOUT: u32 = 40000;
pub const DEFAULT_OUT_OF_SYNC_THRESHOLD: i64 = 15000;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NetworkConfig {
    pub server_address: String,
    pub message_retries_count: Option<u8>,
    pub message_processing_timeout: Option<u32>,
    pub wait_for_timeout: Option<u32>,
    pub out_of_sync_threshold: Option<i64>,
    pub access_key: Option<String>,
}

impl NetworkConfig {
    pub fn server_address(&self) -> &str {
        &self.server_address
    }

    pub fn message_retries_count(&self) -> u8 {
        self.message_retries_count.unwrap_or(DEFAULT_RETRIES_COUNT)
    }

    pub fn message_processing_timeout(&self) -> u32 {
        self.message_processing_timeout.unwrap_or(DEFAULT_PROCESSING_TIMEOUT)
    }

    pub fn wait_for_timeout(&self) -> u32 {
        self.wait_for_timeout.unwrap_or(DEFAULT_WAIT_TIMEOUT)
    }

    pub fn out_of_sync_threshold(&self) -> i64 {
        self.out_of_sync_threshold.unwrap_or(DEFAULT_OUT_OF_SYNC_THRESHOLD)
    }

    pub fn access_key(&self) -> Option<&str> {
        self.access_key.as_ref().map(|string| string.as_str())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SortDirection {
    #[serde(rename = "ASC")]
    Ascending,
    #[serde(rename = "DESC")]
    Descending
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderBy {
    pub path: String,
    pub direction: SortDirection
}

#[derive(Debug, Clone, Serialize)]
pub struct MutationRequest {
    pub id: String,
    pub body: String
}

struct ServerInfo {
    pub version: u64,
    pub supports_time: bool
}

impl ServerInfo {
    pub fn from_version(version: &str) -> ton_types::Result<Self> {
        let mut vec: Vec<&str> = version.split(".").collect();
        vec.resize(3, "0");
        let version = u64::from_str_radix(vec[0], 10)? * 1000000
            + u64::from_str_radix(vec[1], 10)? * 1000
            + u64::from_str_radix(vec[2], 10)?;

        Ok(ServerInfo {
            version,
            supports_time: version >= 26003,
        })
    }
}

struct VariableRequest {
    pub query: String,
    pub variables: Value
}

struct InitedClientData {
    pub query_url: String,
    pub subscription_url: String,
    pub server_info: ServerInfo
}

pub(crate) struct Subscription{
    pub unsubscribe: Pin<Box<dyn Future<Output=()> + Send>>,
    pub data_stream: Pin<Box<dyn Stream<Item=ApiResult<Value>> + Send>>
}

pub(crate) struct NodeClient {
    config: NetworkConfig,
    client_env: Arc<dyn ClientEnv + Send + Sync>,
    data: tokio::sync::RwLock<Option<InitedClientData>>,
    // TODO: use tokio::sync:RwLock when SDK core is fully async
    query_url: std::sync::RwLock<Option<String>>,
}

impl NodeClient {

    pub fn new(config: NetworkConfig, client_env: Arc<dyn ClientEnv + Send + Sync>) -> Self {
        NodeClient {
            config,
            client_env,
            query_url: std::sync::RwLock::new(None),
            data: tokio::sync::RwLock::new(None)
        }
    }

    async fn check_redirect(&self, address: &str) -> ApiResult<String> {
        let result = self.client_env.fetch(
            address,
            FetchMethod::Get,
            None,
            None,
            None
        ).await?;
        
        Ok(result.url)
    }

    fn expand_address(base_url: &str) -> String {
        let base_url =  if  base_url.starts_with("http://") ||
                            base_url.starts_with("https://")
        {
            base_url.to_owned()
        } else {
            format!("https://{}", base_url)
        };

        format!("{}/graphql", base_url)
    }

    async fn query_by_url(&self, address: &str, query: &str) -> ApiResult<Value> {
        let response = self.client_env.fetch(
            &format!("{}?query={}", address, query),
            FetchMethod::Get,
            None,
            None,
            None
        ).await?;

        response.body_as_json()
    }

    async fn query_server_info(&self, address: &str) -> ApiResult<ServerInfo> {
        let response = self.query_by_url(address, "%7Binfo%7Bversion%7D%7D").await?;
        let version = response["data"]["info"]["version"]
            .as_str()
            .ok_or(Error::invalid_server_response(
                format!("No version in response: {}", response)))?;

        ServerInfo::from_version(version)
            .map_err(|err| 
                Error::invalid_server_response(
                    format!("Can not parse version {}: {}", version, err)))
    }

    async fn get_time_delta(&self, address: &str) -> ApiResult<i64>{
        let start = self.client_env.now_ms();
        let response = self.query_by_url(address, "%7Binfo%7Btime%7D%7D").await?;
        let end = self.client_env.now_ms();
        let server_time = response["data"]["info"]["time"]
            .as_i64()
            .ok_or(Error::invalid_server_response(format!("No time in response: {}", response)))?;

        Ok(server_time - (start + (end - start) / 2))
    }

    async fn check_time_delta(&self, address: &str, config: &NetworkConfig) -> ApiResult<()> {
        let delta = self.get_time_delta(address).await?;
        if delta.abs() >= config.out_of_sync_threshold() {
            Err(Error::clock_out_of_sync(delta, config.out_of_sync_threshold()))
        } else {
            Ok(())
        }
    }

    async fn init(&self, config: &NetworkConfig) -> ApiResult<InitedClientData> {
        let queries_server = Self::expand_address(config.server_address());
        let redirected = self.check_redirect(&queries_server).await?;
        let queries_server = redirected.clone();
        let subscriptions_server = redirected
            .replace("https://", "wss://")
            .replace("http://", "ws://");

        let server_info = self.query_server_info(&queries_server).await?;
        if server_info.supports_time {
            self.check_time_delta(&queries_server, config).await?;
        }

        Ok(InitedClientData {
            query_url: queries_server,
            subscription_url: subscriptions_server,
            server_info,
        })
    }

    async fn ensure_client(&self) -> ApiResult<()> {
        if self.data.read().await.is_some() {
            return Ok(());
        }

        let mut data = self.data.write().await;
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
        self.config.server_address()
    }

    pub fn query_url(&self) -> Option<String> {
        self.query_url.read().unwrap().clone()
    }

    // Returns Stream with updates database fileds by provided filter
    pub async fn subscribe(
        &self, table: &str, filter: &Value, fields: &str
    ) -> ApiResult<Subscription> {
        let request = Self::generate_subscription(table, filter, fields);

        let mut websocket = {
            self.ensure_client().await?;
            let client_lock = self.data.read().await;
            let address = &client_lock.as_ref().unwrap().subscription_url;

            self.client_env.websocket_connect(
                &address,
                Some(HashMap::from_iter(vec![("Sec-WebSocket-Protocol", "graphql-ws")].into_iter()))
            ).await?
        };
        
        // map stream of strings into GraphQL JSON answers
        let table = table.to_owned();
        let data_stream = websocket.receiver
            .filter_map(move |result| {
                let closure_table = table.clone();
                async move {
                    match result {
                        Err(err) => Some(Err(err)),
                        Ok(value) => {
                            let value: Value = match serde_json::from_str(&value) {
                                Err(err) => return Some(Err(Error::invalid_server_response(
                                    format!("Subscription answer is not a valid JSON: {}\n{}", err, value)))),
                                Ok(value) => value
                            };

                            // skip ack and keep alive messages
                            if value["type"] == "connection_ack" || value["type"] == "ka" {
                                return None;
                            }
                            
                            // try to extract the record value from the answer
                            let record_value = &value["payload"]["data"][&closure_table];

                            if record_value.is_null() {
                                Some(Err(Error::invalid_server_response(
                                    format!("Invalid subscription answer: {}", value))))
                            } else {
                                Some(Ok(record_value.clone()))
                            }
                        }
                    }
                }
            });

        let id = rand::thread_rng().next_u32();

        let mut init_request = serde_json::json!({
            "type": "connection_init",
            "payload": {}
        });
        if let Some(access_key) = &self.config.access_key {
            init_request["payload"]["accessKey"] = access_key.as_str().into();
        }
        websocket.sender.send(init_request.to_string()).await?;

        let request = serde_json::json!({
            "id": id,
            "type": "start",
            "payload": {
                "query": request.query,
                "variables": request.variables,
            }
        }).to_string();
        websocket.sender.send(request).await?;

        let client_env = self.client_env.clone();
        let handle = websocket.handle;
        let mut sender = websocket.sender;
        let unsubscribe = async move {
            let _ = sender.send(
                serde_json::json!({
                    "id": id,
                    "type": "stop",
                    "payload": {}
                }).to_string()
            ).await;
            client_env.websocket_close(handle).await;
        };

        Ok(Subscription {
            data_stream: Box::pin(data_stream),
            unsubscribe: Box::pin(unsubscribe)
        })
    }

    // Returns Stream with required database record fields
    pub async fn load_record_fields(&self, table: &str, record_id: &str, fields: &str)
        -> ApiResult<Value> {
        let value = self.query(
            table,
            &serde_json::json!({ 
                "id": { "eq": record_id } 
            }),
            fields,
            None,
            None,
            None).await?;

        Ok(value[0].clone())
    }

    pub fn try_extract_error(value: &Value) -> Option<ApiError> {
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
                                return Some(Error::graphql_error(string))
                            }
                        }
                    }
                }
            }
        }
    
        return None;
    }

    async fn query_vars(
        &self, address: &str, request: VariableRequest, timeout: Option<u32>
    ) -> ApiResult<Value> {
        let request = json!({
            "query": request.query,
            "variables": request.variables,
        }).to_string();

        let mut headers = HashMap::new();
        headers.insert("content-type".to_owned(), "application/json".to_owned());

        let response = self.client_env.fetch(
            address,
            FetchMethod::Post,
            Some(headers),
            Some(request.into_bytes()),
            timeout,
        ).await?;

        let response = response.body_as_json()?;

        if let Some(error) = Self::try_extract_error(&response) {
            Err(error)
        } else {
            Ok(response)
        }
    }

    // Returns Stream with GraphQL query answer
    pub async fn query(
        &self,
        table: &str,
        filter: &Value,
        fields: &str,
        order_by: Option<Vec<OrderBy>>,
        limit: Option<u32>,
        timeout: Option<u32>
    ) -> ApiResult<Value> {
        let query = Self::generate_query_var(table, filter, fields, order_by, limit, timeout);

        self.ensure_client().await?;
        let client_lock = self.data.read().await;
        let address = &client_lock.as_ref().unwrap().query_url;

        let result = self.query_vars(
            address, query, timeout).await?;

        // try to extract the record value from the answer
        let records_array = &result["data"][&table];
        if records_array.is_null() {
            Err(Error::invalid_server_response(format!("Invalid query answer: {}", result)))
        } else {
            Ok(records_array.clone())
        }
    }

    // Executes GraphQL query, waits for result and returns recieved value
    pub async fn wait_for(&self, table: &str, filter: &Value, fields: &str, timeout: Option<u32>)
        -> ApiResult<Value>
    {
        let value = self.query(
            table,
            filter,
            fields,
            None,
            None,
            timeout.or(Some(self.config.wait_for_timeout()))).await?;

        if !value[0].is_null() {
            Ok(value[0].clone())
        } else {
            Err(Error::wait_for_timeout())
        }
    }

    fn generate_query_var(
        table: &str,
        filter: &Value,
        fields: &str,
        order_by: Option<Vec<OrderBy>>,
        limit: Option<u32>,
        timeout: Option<u32>
    ) -> VariableRequest {
        let mut scheme_type: Vec<String> = table.split_terminator("_")
            .map(|word| {
                let mut word = word.to_owned();
                word[..1].make_ascii_uppercase();
                word
            })
            .collect();
        scheme_type[0] = scheme_type[0].trim_end_matches("s").to_owned();
        let scheme_type: String = scheme_type.join("") + "Filter";

        let mut query = format!(
            r#"query {table}
            ($filter: {scheme_type}, $orderBy: [QueryOrderBy], $limit: Int, $timeout: Float)
            {{
                {table}(filter: $filter, orderBy: $orderBy, limit: $limit, timeout: $timeout)
                {{ {fields} }}
            }}"#,
            table=table,
            scheme_type=scheme_type,
            fields=fields
        );
        query = query.split_whitespace().collect::<Vec<&str>>().join(" ");

        let variables = json!({
            "filter" : filter,
            "orderBy": order_by,
            "limit": limit,
            "timeout": timeout
        });

        VariableRequest {
            query,
            variables
        }
    }

    fn generate_subscription(table: &str, filter: &Value, fields: &str) -> VariableRequest {
        let mut scheme_type = (&table[0 .. table.len() - 1]).to_owned() + "Filter";
        scheme_type[..1].make_ascii_uppercase();

        let query = format!("subscription {table}($filter: {type}) {{ {table}(filter: $filter) {{ {fields} }} }}",
            type=scheme_type,
            table=table,
            fields=fields);
        let query = query.split_whitespace().collect::<Vec<&str>>().join(" ");

        let variables = json!({
            "filter" : filter,
        });

        VariableRequest {
            query,
            variables
        }
    }

    fn generate_post_mutation(requests: &[MutationRequest]) -> VariableRequest {
        let query = "mutation postRequests($requests:[Request]){postRequests(requests:$requests)}".to_owned();
        let variables = json!({
            "requests": serde_json::json!(requests)
        });

        VariableRequest {
            query,
            variables
        }
    }

    // Sends message to node
    pub async fn send_message(&self, key: &[u8], value: &[u8]) -> ApiResult<()> {
        let request = MutationRequest {
            id: base64::encode(key),
            body: base64::encode(value)
        };

        self.ensure_client().await?;
        let client_lock = self.data.read().await;
        let address = &client_lock.as_ref().unwrap().query_url;

        let result = self.query_vars(
            address,
            Self::generate_post_mutation(&[request]),
            None
        ).await;

        // send message is always successful in order to process case when server received message
        // but client didn't receive responce
        if let Err(err) = result {
            log::warn!("Post message error: {}", err.message);
        }

        Ok(())
    }
}
