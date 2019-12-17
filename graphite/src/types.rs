/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
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

use futures::{Async, Poll};
use futures::stream::Stream;
use std::fmt;
use reqwest::{Response, RequestBuilder};
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

pub struct PeriodicRequestStream {
    request: RequestBuilder,
    timeout: u64
}

impl PeriodicRequestStream {
    pub fn new(request: RequestBuilder) -> Result<Self, GraphiteError> {
        Ok(Self { request, timeout: 0 })
    }
}

impl Stream for PeriodicRequestStream {
    type Item = Value;
    type Error = GraphiteError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        std::thread::sleep(std::time::Duration::from_millis(self.timeout));
        self.timeout = 1000;

        match  self.request.try_clone().unwrap().send() {
            Ok(mut res) => {
                match res.text() {
                    Ok(res_str) => {
                        if let Ok(value) = serde_json::from_str(res_str.as_str()) {
                            if let Some(error) = try_extract_error(&value) {
                                return Err(error);
                            }
                            Ok(Async::Ready(Some(value)))
                        } else {
                            Err(GraphiteError::new(format!(
                                        "Invalid JSON: {}", res_str)))
                        }
                    },
                    Err(err) => Err(GraphiteError::new(err.to_string().clone()))
                }
            },
            Err(err) => Err(GraphiteError::new(err.to_string().clone()))
        }
    }
}

pub struct ResponseStream {
    response: Option<Result<Response, reqwest::Error>>
}

impl ResponseStream {
    pub fn new(response: Result<Response, reqwest::Error>) -> Result<Self, GraphiteError> {
        Ok(Self { response: Some(response) })
    }
}

impl Stream for ResponseStream {
    type Item = Value;
    type Error = GraphiteError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match &mut self.response.take() {
            Some(response) => {
                match response {
                    Ok(res) => {
                        match res.text() {
                            Ok(res_str) => {
                                if let Ok(value) = serde_json::from_str(res_str.as_str()) {
                                    if let Some(error) = try_extract_error(&value) {
                                        return Err(error);
                                    }
                                    Ok(Async::Ready(Some(value)))
                                } else {
                                    Err(GraphiteError::new(format!(
                                        "Invalid JSON: {}", res_str)))
                                }
                            },
                            Err(err) => Err(GraphiteError::new(err.to_string().clone()))
                        }
                    },
                    Err(err) => Err(GraphiteError::new(err.to_string().clone()))
                }
            },
            None => Ok(Async::Ready(None))
        }
    }
}

pub struct SubscribeStream {
    id: u64,
    request: VariableRequest,
    client: Client<Box<dyn NetworkStream + Send>>
}

impl SubscribeStream {
    pub fn new(id: u64, request: VariableRequest, host:&str) -> Result<Self, GraphiteError> {
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

    pub fn get_id(&self) -> u64 {
        self.id.clone()
    }
}

impl Drop for SubscribeStream {
    fn drop(&mut self) {
        let _ = self.unsubscribe();
    }
}

fn try_extract_error(value: &Value) -> Option<GraphiteError> {
    if let Some(payload) = value.get("payload") {
        if let Some(errors) = payload.get("errors") {
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
    }
    return None;
}

impl Stream for SubscribeStream {
    type Item = Value;
    type Error = GraphiteError;


    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Some(result) = self.client.incoming_messages().next() {
            match result {
                Ok(message) => {
                    match message {
                        OwnedMessage::Text(text) => {
                            if let Ok(value) = serde_json::from_str(text.as_str()) {
                                if let Some(error) = try_extract_error(&value) {
                                    return Err(error);
                                }
                                Ok(Async::Ready(Some(value)))

                            } else {
                                Err(GraphiteError::new(format!(
                                        "Invalid JSON: {}", text)))
                            }
                        },
                        _ => Ok(Async::NotReady)
                    }

                } ,
                Err(err) => Err(GraphiteError::new(err.to_string())),
            }
        } else {
            Ok(Async::NotReady)
        }
    }
}
