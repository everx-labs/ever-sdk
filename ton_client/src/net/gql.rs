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

use crate::error::{ClientError, ClientResult};
use crate::net::Error;
use serde_json::Value;

const GQL_CONNECTION_INIT: &str = "connection_init";
const GQL_CONNECTION_ACK: &str = "connection_ack";
const GQL_CONNECTION_ERROR: &str = "connection_error";
const GQL_CONNECTION_KEEP_ALIVE: &str = "ka";
// const GQL_CONNECTION_TERMINATE: &str = "connection_terminate";
const GQL_START: &str = "start";
const GQL_DATA: &str = "data";
const GQL_ERROR: &str = "error";
const GQL_COMPLETE: &str = "complete";
const GQL_STOP: &str = "stop";

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
pub struct PostRequest {
    pub id: String,
    pub body: String,
}

#[derive(Debug)]
pub(crate) enum GraphQLMessageFromClient {
    ConnectionInit {
        connection_params: Value,
    },
    // ConnectionTerminate,
    Start {
        id: String,
        query: String,
        variables: Option<Value>,
        operation_name: Option<String>,
    },
    Stop {
        id: String,
    },
}

impl GraphQLMessageFromClient {
    pub fn get_message(&self) -> String {
        match self {
            GraphQLMessageFromClient::ConnectionInit { connection_params } => json!({
                "type": GQL_CONNECTION_INIT,
                "payload": connection_params.clone(),
            }),
            // GraphQLMessageFromClient::ConnectionTerminate => json!({
            //     "type": GQL_CONNECTION_TERMINATE,
            // }),
            GraphQLMessageFromClient::Start {
                id,
                query,
                variables,
                operation_name,
            } => {
                let mut payload = json!({
                    "query": query.clone(),
                });
                if let Some(variables) = variables {
                    payload["variables"] = variables.clone();
                }
                if let Some(operation_name) = operation_name {
                    payload["operationName"] = Value::String(operation_name.clone());
                }
                json!({
                    "type": GQL_START,
                    "id": id,
                    "payload": payload,
                })
            }
            GraphQLMessageFromClient::Stop { id } => json!({
                "type": GQL_STOP,
                "id": id,
            }),
        }.to_string()
    }
}

#[derive(Debug)]
pub(crate) enum GraphQLMessageFromServer {
    ConnectionError {
        error: Value,
    },
    ConnectionAck,
    ConnectionKeepAlive,
    Data {
        id: String,
        data: Value,
        errors: Option<Vec<Value>>,
    },
    Error {
        id: String,
        error: Value,
    },
    Complete {
        id: String,
    },
}

impl GraphQLMessageFromServer {
    pub fn parse(message: &str) -> ClientResult<Self> {
        let value = serde_json::from_str::<Value>(message).map_err(|_| Error::invalid_server_response(message))?;
        Ok(match value["type"].as_str().unwrap_or("") {
            GQL_CONNECTION_ERROR => GraphQLMessageFromServer::ConnectionError {
                error: value["payload"].clone(),
            },
            GQL_CONNECTION_ACK => GraphQLMessageFromServer::ConnectionAck,
            GQL_CONNECTION_KEEP_ALIVE => GraphQLMessageFromServer::ConnectionKeepAlive,
            GQL_DATA => GraphQLMessageFromServer::Data {
                id: value["id"].as_str().unwrap_or("").to_string(),
                data: value["payload"]["data"].clone(),
                errors: value["payload"]["errors"].as_array().cloned(),
            },
            GQL_ERROR => GraphQLMessageFromServer::Error {
                id: value["id"].as_str().unwrap_or("").to_string(),
                error: value["payload"].clone(),
            },
            GQL_COMPLETE => GraphQLMessageFromServer::Complete {
                id: value["id"].as_str().unwrap_or("").to_string(),
            },
            _ => return Err(Error::invalid_server_response(message)),
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GraphQLOperation {
    pub query: String,
    pub variables: Option<Value>,
    pub operation_name: Option<String>,
}

impl GraphQLOperation {
    pub fn get_start_message(&self, id: String) -> GraphQLMessageFromClient {
        GraphQLMessageFromClient::Start {
            id,
            query: self.query.clone(),
            variables: self.variables.clone(),
            operation_name: self.operation_name.clone(),
        }
    }

    pub fn query(
        table: &str,
        filter: &Value,
        fields: &str,
        order_by: Option<Vec<OrderBy>>,
        limit: Option<u32>,
        timeout: Option<u32>,
    ) -> Self {
        let mut scheme_type: Vec<String> = table.split_terminator("_").map(|word| {
            let mut word = word.to_owned();
            word[..1].make_ascii_uppercase();
            word
        }).collect();
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

        Self {
            query,
            variables: Some(variables),
            operation_name: None,
        }
    }

    pub fn subscription(
        table: &str,
        filter: &Value,
        fields: &str,
    ) -> Self {
        let mut scheme_type = (&table[0..table.len() - 1]).to_owned() + "Filter";
        scheme_type[..1].make_ascii_uppercase();

        let query = format!("subscription {table}($filter: {type}) {{ {table}(filter: $filter) {{ {fields} }} }}",
            type=scheme_type,
            table=table,
            fields=fields);
        let query = query.split_whitespace().collect::<Vec<&str>>().join(" ");
        let variables = Some(json!({
            "filter" : filter,
        }));
        Self {
            query,
            variables,
            operation_name: None,
        }
    }

    pub fn post_requests(requests: &[PostRequest]) -> Self {
        let query = "mutation postRequests($requests:[Request]){postRequests(requests:$requests)}".to_owned();
        let variables = Some(json!({ "requests": serde_json::json!(requests) }));
        Self {
            query,
            variables,
            operation_name: None,
        }
    }
}

#[derive(Debug)]
pub enum GraphQLOperationEvent {
    Id(u32),
    Data(Value),
    Error(ClientError),
    Complete,
}
