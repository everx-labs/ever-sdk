/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

extern crate futures;
extern crate websocket;

use futures::task::{Poll, Context};
use futures::stream::Stream;
use std::fmt;
use serde_json::Value;
use websocket::{ClientBuilder, OwnedMessage};
use websocket::client::sync::Client;
use websocket::stream::sync::NetworkStream;


#[derive(Debug, Clone)]
pub struct GraphiteError {
    message: String,
}

impl GraphiteError {
    pub fn new(message: String) -> Self {
        Self { message: message }
    }
}

impl fmt::Display for GraphiteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({})", self.message)
    }
}

impl std::error::Error for GraphiteError {}

pub struct VariableRequest {
    query: String,
    variables: Option<String>
}

impl VariableRequest {
    pub fn new(query: String, variables: Option<String>) -> Self {
        Self {
            query, variables
        }
    }

    pub fn get_query(&self) -> String {
        self.query.clone()
    }

    pub fn get_variables(&self) -> Option<String> {
        self.variables.clone()
    }
}

pub struct SubscribeStream {
    id: u32,
    request: VariableRequest,
    client: Client<Box<dyn NetworkStream + Send>>
}

impl SubscribeStream {
    pub fn new(id: u32, request: VariableRequest, host:&str) -> Result<Self, GraphiteError> {
        let client = ClientBuilder::new(host)
            .map_err(|err| 
                GraphiteError::new(
                    format!("Can't create websocket client with address {}. Error {}", host, err)))?
            .add_protocol("graphql-ws")
            .connect(None)
            .map_err(|err|
                GraphiteError::new(
                    format!("Can't connect to websocket server {}. Error {}", host, err)))?;

        let mut future = Self {
            id: id,
            request: request,
            client
        };

        future.subscribe()?;
        Ok(future)
    }

    pub fn subscribe(&mut self) -> Result<(), GraphiteError> {
        let query = &self.request.get_query().clone();
        let variables = &self.request.get_variables().clone();
        let request: String;

        if let Some(vars) = variables {
            request = format!("{{\"id\":{}, \"type\": \"start\", \"payload\":{{ \"query\": \"{}\", \"variables\": {} }}}}", &self.id, &query, &vars);
        } else {
            request = format!("{{\"id\":{}, \"type\": \"start\", \"payload\":{{ \"query\": \"{}\" }}}}", &self.id, &query);
        }

        let msg = OwnedMessage::Text(request);
        self.client.send_message(&msg)
            .map_err(|err| 
                GraphiteError::new(
                    format!("Sending message across stdin channel failed. Error: {}", err)))?;

        Ok(())
    }

    pub fn unsubscribe(&mut self) -> Result<(), GraphiteError> {
        let query = format!("{{\"id\":{}, \"type\": \"stop\", \"payload\":{{}}}}", &self.id);
        let msg = OwnedMessage::Text(query.to_string());
        self.client.send_message(&msg)
            .map_err(|err| 
                GraphiteError::new(
                    format!("Sending message across stdin channel failed. Error: {}", err)))?;

        Ok(())
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
}

impl Drop for SubscribeStream {
    fn drop(&mut self) {
        let _ = self.unsubscribe();
    }
}

pub fn try_extract_error(value: &Value) -> Option<GraphiteError> {
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
                            return Some(GraphiteError::new(string.to_string()))
                        }
                    }
                }
            }
        }
    }

    return None;
}

impl Stream for SubscribeStream {
    type Item = Result<Value, GraphiteError>;

    fn poll_next(self: std::pin::Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(result) = self.get_mut().client.incoming_messages().next() {
            match result {
                Ok(message) => {
                    match message {
                        OwnedMessage::Text(text) => {
                            if let Ok(value) = serde_json::from_str(text.as_str()) {
                                if let Some(error) = try_extract_error(&value) {
                                    return Poll::Ready(Some(Err(error)));
                                }
                                Poll::Ready(Some(Ok(value)))
                            } else {
                                Poll::Ready(Some(Err(GraphiteError::new(format!(
                                        "Invalid JSON: {}", text)))))
                            }
                        },
                        _ => Poll::Pending
                    }

                } ,
                Err(err) => Poll::Ready(Some(Err(GraphiteError::new(err.to_string())))),
            }
        } else {
            Poll::Pending
        }
    }
}
