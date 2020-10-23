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
use crate::net::Error;
use futures::{Future, SinkExt, Stream, StreamExt};
use rand::RngCore;
use serde_json::Value;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::pin::Pin;
use std::sync::Arc;
use ton_sdk::NetworkConfig;

pub const MAX_TIMEOUT: u32 = std::i32::MAX as u32;

#[derive(Serialize, Deserialize, Clone, ApiType)]
pub enum SortDirection {
    ASC,
    DESC,
}

#[derive(Serialize, Deserialize, Clone, ApiType)]
pub struct OrderBy {
    pub path: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize)]
struct MutationRequest {
    pub id: String,
    pub body: String,
}

struct ServerVersion {
    pub version: u64,
    pub supports_time: bool,
}

impl ServerVersion {
    pub fn from_version(version: &str) -> ton_types::Result<Self> {
        let mut vec: Vec<&str> = version.split(".").collect();
        vec.resize(3, "0");
        let version = u64::from_str_radix(vec[0], 10)? * 1000000
            + u64::from_str_radix(vec[1], 10)? * 1000
            + u64::from_str_radix(vec[2], 10)?;

        Ok(ServerVersion {
            version,
            supports_time: version >= 26003,
        })
    }
}

struct VariableRequest {
    pub query: String,
    pub variables: Value,
}

struct ServerInfo {
    pub query_url: String,
    pub subscription_url: String,
    pub server_version: ServerVersion,
}

pub(crate) struct Subscription {
    pub unsubscribe: Pin<Box<dyn Future<Output = ()> + Send>>,
    pub data_stream: Pin<Box<dyn Stream<Item = ClientResult<Value>> + Send>>,
}

pub(crate) struct NodeClient {
    config: NetworkConfig,
    client_env: Arc<ClientEnv>,
    server_info: tokio::sync::RwLock<Option<ServerInfo>>,
    // TODO: use tokio::sync:RwLock when SDK core is fully async
    query_url: std::sync::RwLock<Option<String>>,
}

impl NodeClient {
    pub fn new(config: NetworkConfig, client_env: Arc<ClientEnv>) -> Self {
        NodeClient {
            config,
            client_env,
            query_url: std::sync::RwLock::new(None),
            server_info: tokio::sync::RwLock::new(None),
        }
    }

    fn expand_address(base_url: &str) -> String {
        let base_url = if base_url.starts_with("http://") || base_url.starts_with("https://") {
            base_url.to_owned()
        } else {
            format!("https://{}", base_url)
        };

        format!("{}/graphql", base_url)
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

    async fn get_server_info(&self, address: &str) -> ClientResult<ServerInfo> {
        let response = self
            .client_env
            .fetch(
                &format!("{}?query=%7Binfo%7Bversion%7D%7D", address),
                FetchMethod::Get,
                None,
                None,
                None,
            )
            .await?;
        let response_body = response.body_as_json()?;

        let version = response_body["data"]["info"]["version"].as_str().ok_or(
            Error::invalid_server_response(format!("No version in response: {}", response_body)),
        )?;

        let server_version = ServerVersion::from_version(version).map_err(|err| {
            Error::invalid_server_response(format!("Can not parse version {}: {}", version, err))
        })?;

        let query_url = response
            .url
            .trim_end_matches("?query=%7Binfo%7Bversion%7D%7D")
            .to_owned();
        let subscription_url = query_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");

        Ok(ServerInfo {
            query_url,
            subscription_url,
            server_version,
        })
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
        if delta.abs() >= config.out_of_sync_threshold {
            Err(Error::clock_out_of_sync(
                delta,
                config.out_of_sync_threshold,
            ))
        } else {
            Ok(())
        }
    }

    async fn init(&self, config: &NetworkConfig) -> ClientResult<ServerInfo> {
        let queries_server = Self::expand_address(&config.server_address);
        let server_info = self.get_server_info(&queries_server).await?;

        if server_info.server_version.supports_time {
            self.check_time_delta(&queries_server, config).await?;
        }

        Ok(server_info)
    }

    async fn ensure_client(&self) -> ClientResult<()> {
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
        let request = Self::generate_subscription(table, filter, fields);

        let mut websocket = {
            self.ensure_client().await?;
            let client_lock = self.server_info.read().await;
            let address = &client_lock.as_ref().unwrap().subscription_url;

            self.client_env
                .websocket_connect(
                    &address,
                    Some(HashMap::from_iter(
                        vec![("Sec-WebSocket-Protocol".to_owned(), "graphql-ws".to_owned())]
                            .into_iter(),
                    )),
                )
                .await?
        };

        // map stream of strings into GraphQL JSON answers
        let table = table.to_owned();
        let data_stream = websocket.receiver.filter_map(move |result| {
            let closure_table = table.clone();
            async move {
                match result {
                    Err(err) => Some(Err(err)),
                    Ok(value) => {
                        let value: Value = match serde_json::from_str(&value) {
                            Err(err) => {
                                return Some(Err(Error::invalid_server_response(format!(
                                    "Subscription answer is not a valid JSON: {}\n{}",
                                    err, value
                                ))))
                            }
                            Ok(value) => value,
                        };

                        // skip ack and keep alive messages
                        if value["type"] == "connection_ack" || value["type"] == "ka" {
                            return None;
                        }

                        // try to extract the record value from the answer
                        let record_value = &value["payload"]["data"][&closure_table];

                        if record_value.is_null() {
                            Some(Err(Error::invalid_server_response(format!(
                                "Invalid subscription answer: {}",
                                value
                            ))))
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
        })
        .to_string();
        websocket.sender.send(request).await?;

        let mut sender = websocket.sender;
        let unsubscribe = async move {
            let _ = sender
                .send(
                    serde_json::json!({
                        "id": id,
                        "type": "stop",
                        "payload": {}
                    })
                    .to_string(),
                )
                .await;
        };

        Ok(Subscription {
            data_stream: Box::pin(data_stream),
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

    async fn query_vars(
        &self,
        address: &str,
        request: VariableRequest,
        timeout: Option<u32>,
    ) -> ClientResult<Value> {
        let request = json!({
            "query": request.query,
            "variables": request.variables,
        })
        .to_string();

        let mut headers = HashMap::new();
        headers.insert("content-type".to_owned(), "application/json".to_owned());

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
    pub async fn query(
        &self,
        table: &str,
        filter: &Value,
        fields: &str,
        order_by: Option<Vec<OrderBy>>,
        limit: Option<u32>,
        timeout: Option<u32>,
    ) -> ClientResult<Value> {
        let query = Self::generate_query_var(table, filter, fields, order_by, limit, timeout);

        self.ensure_client().await?;
        let client_lock = self.server_info.read().await;
        let address = &client_lock.as_ref().unwrap().query_url;

        let result = self.query_vars(address, query, timeout).await?;

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

    // Executes GraphQL query, waits for result and returns recieved value
    pub async fn wait_for(
        &self,
        table: &str,
        filter: &Value,
        fields: &str,
        timeout: Option<u32>,
    ) -> ClientResult<Value> {
        let value = self
            .query(
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

    fn generate_query_var(
        table: &str,
        filter: &Value,
        fields: &str,
        order_by: Option<Vec<OrderBy>>,
        limit: Option<u32>,
        timeout: Option<u32>,
    ) -> VariableRequest {
        let mut scheme_type: Vec<String> = table
            .split_terminator("_")
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
            table = table,
            scheme_type = scheme_type,
            fields = fields
        );
        query = query.split_whitespace().collect::<Vec<&str>>().join(" ");

        let variables = json!({
            "filter" : filter,
            "orderBy": order_by,
            "limit": limit,
            "timeout": timeout
        });

        VariableRequest { query, variables }
    }

    fn generate_subscription(table: &str, filter: &Value, fields: &str) -> VariableRequest {
        let mut scheme_type = (&table[0..table.len() - 1]).to_owned() + "Filter";
        scheme_type[..1].make_ascii_uppercase();

        let query = format!("subscription {table}($filter: {type}) {{ {table}(filter: $filter) {{ {fields} }} }}",
            type=scheme_type,
            table=table,
            fields=fields);
        let query = query.split_whitespace().collect::<Vec<&str>>().join(" ");

        let variables = json!({
            "filter" : filter,
        });

        VariableRequest { query, variables }
    }

    fn generate_post_mutation(requests: &[MutationRequest]) -> VariableRequest {
        let query = "mutation postRequests($requests:[Request]){postRequests(requests:$requests)}"
            .to_owned();
        let variables = json!({ "requests": serde_json::json!(requests) });

        VariableRequest { query, variables }
    }

    // Sends message to node
    pub async fn send_message(&self, key: &[u8], value: &[u8]) -> ClientResult<()> {
        let request = MutationRequest {
            id: base64::encode(key),
            body: base64::encode(value),
        };

        self.ensure_client().await?;
        let client_lock = self.server_info.read().await;
        let address = &client_lock.as_ref().unwrap().query_url;

        let result = self
            .query_vars(address, Self::generate_post_mutation(&[request]), None)
            .await;

        // send message is always successful in order to process case when server received message
        // but client didn't receive responce
        if let Err(err) = result {
            log::warn!("Post message error: {}", err.message);
        }

        Ok(())
    }
}
