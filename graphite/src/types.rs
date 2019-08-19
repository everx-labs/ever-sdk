extern crate futures;
extern crate websocket;

use futures::{Async, Poll};
use futures::stream::Stream;
use std::fmt;
use reqwest::Response;
use serde_json::Value;
use std::net::TcpStream;
use websocket::{ClientBuilder, OwnedMessage};
use websocket::receiver::Reader;
use websocket::sender::Writer;


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

pub struct ResponseStream {
    response: Option<Result<Response, reqwest::Error>>
}

impl ResponseStream {
    pub fn new(response: Result<Response, reqwest::Error>) -> Self {
        Self { response: Some(response) }
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
                                    Err(GraphiteError::new("Invalid JSON".to_string()))
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
    receiver: Reader<TcpStream>,
    sender: Writer<TcpStream>
}

impl SubscribeStream {
    pub fn new(id: u64, request: VariableRequest, host: &str) -> Self {
        let client = ClientBuilder::new(host)
            .unwrap()
            .add_protocol("graphql-ws")
            .connect_insecure()
            .unwrap();
        let (receiver, sender) = client.split().unwrap();

        let mut future = Self {
            id: id,
            request: request,
            receiver: receiver,
            sender: sender
        };

        future.subscribe();
        return future;
    }

    pub fn subscribe(&mut self) {
        let query = &self.request.get_query().clone();
        let variables = &self.request.get_variables().clone();
        let request: String;

        if let Some(vars) = variables {
            request = format!("{{\"id\":{}, \"type\": \"start\", \"payload\":{{ \"query\": \"{}\", \"variables\": {} }}}}", &self.id, &query, &vars);
        } else {
            request = format!("{{\"id\":{}, \"type\": \"start\", \"payload\":{{ \"query\": \"{}\" }}}}", &self.id, &query);
        }

        let msg = OwnedMessage::Text(request);
        self.sender.send_message(&msg).expect("Sending message across stdin channel.");
    }

    pub fn unsubscribe(id: u64, sender: &mut Writer<TcpStream>) {
        let query = format!("{{\"id\":{}, \"type\": \"stop\", \"payload\":{{}}}}", &id);
        let msg = OwnedMessage::Text(query.to_string());
        sender.send_message(&msg).expect("Sending message across stdin channel.");
    }

    pub fn get_id(&self) -> u64 {
        self.id.clone()
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
        if let Some(result) = self.receiver.incoming_messages().next() {
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
                                Err(GraphiteError::new("Invalid JSON".to_string()))
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
