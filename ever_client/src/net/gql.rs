/*
 * Copyright 2018-2021 EverX Labs Ltd.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific EVERX DEV software governing permissions and
 * limitations under the License.
 *
 */

use serde_json::Value;

use crate::error::{ClientResult};
use crate::net::Error;

const GQL_CONNECTION_INIT: &str = "connection_init";
const GQL_CONNECTION_ACK: &str = "connection_ack";
const GQL_CONNECTION_ERROR: &str = "connection_error";
const GQL_CONNECTION_KEEP_ALIVE: &str = "ka";
const GQL_CONNECTION_TERMINATE: &str = "connection_terminate";
const GQL_START: &str = "start";
const GQL_DATA: &str = "data";
const GQL_ERROR: &str = "error";
const GQL_COMPLETE: &str = "complete";
const GQL_STOP: &str = "stop";

#[derive(Debug)]
pub(crate) enum GraphQLMessageFromClient {
    ConnectionInit {
        connection_params: Value,
    },
    ConnectionTerminate,
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
            GraphQLMessageFromClient::ConnectionTerminate => json!({
                "type": GQL_CONNECTION_TERMINATE,
            }),
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
        }
        .to_string()
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
        let value = serde_json::from_str::<Value>(message)
            .map_err(|_| Error::invalid_server_response(message))?;
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

