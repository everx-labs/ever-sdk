extern crate futures;
extern crate websocket;

use futures::{Async, Poll};
use futures::stream::Stream;
use std::fmt;
use reqwest::Response;
use serde_json::Value;
use std::error::Error;
use std::net::TcpStream;
use websocket::{ClientBuilder, OwnedMessage};
use websocket::receiver::Reader;
use websocket::result::WebSocketError;
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

pub struct ResponseStream {
    response: Result<Response, reqwest::Error>
}

impl ResponseStream {
    pub fn new(response: Result<Response, reqwest::Error>) -> Self {
        Self { response }
    }
}

impl Stream for ResponseStream {
    type Item = Value;
    type Error = GraphiteError;
    
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match &mut self.response {
            Ok(res) => {
                match res.text() {
                    Ok(res_str) => Ok(Async::Ready(Some(Value::from(res_str.clone())))),
                    Err(err) => Err(GraphiteError::new(err.to_string().clone()))
                }
            },
            Err(err) => Err(GraphiteError::new(err.to_string().clone()))
        }
    }
}

pub struct SubscribeStream {
    id: u64,
    query: String,
    receiver: Reader<TcpStream>,
    sender: Writer<TcpStream>
}

impl SubscribeStream {
    pub fn new(id: u64, query: String, host: &str) -> Self {  
        let client = ClientBuilder::new(host)
            .unwrap()
            .add_protocol("graphql-ws")
            .connect_insecure()
            .unwrap();   
        let (receiver, sender) = client.split().unwrap();
               
        let mut future = Self {
            id: id,
            query: query,
            receiver: receiver,
            sender: sender
        };
        
        future.subscribe();    
        return future;
    }
    
    pub fn subscribe(&mut self) {
        let query = format!("{{\"id\":{}, \"type\": \"start\", \"payload\":{{ \"query\": \"{}\" }}}}", &self.id, &self.query);
        let msg = OwnedMessage::Text(query.to_string());
        self.sender.send_message(&msg).expect("Sending message across stdin channel.");
    }
    
    pub fn unsubscribe(id: u64, sender: &mut Writer<TcpStream>) {
        let query = format!("{{\"id\":{}, \"type\": \"stop\", \"payload\":{{}}}}", &id);
        let msg = OwnedMessage::Text(query.to_string());
        sender.send_message(&msg).expect("Sending message across stdin channel.");
    }
}

impl Stream for SubscribeStream {
    type Item = Value;
    type Error = GraphiteError;
    
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Some(result) = self.receiver.incoming_messages().next() {
            match result {
                Ok(message) => {
                    match message {
                        OwnedMessage::Text(text) => Ok(Async::Ready(Some(Value::from(text)))),
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