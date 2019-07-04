use crate::types::{ResponseStream, SubscribeStream};

use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use std::net::TcpStream;
use websocket::ClientBuilder;
use websocket::sender::Writer;


pub struct GqlClient {
    client: Client,
    socket_sender: Writer<TcpStream>,
    graphql_host: String,
    graphql_socket_host: String,
    incremented_id: u64
}

impl GqlClient {
    pub fn new(graphql_host: String, graphql_socket_host: String) -> Self {
        let client = ClientBuilder::new(&graphql_socket_host)
            .unwrap()
            .add_protocol("graphql-ws")
            .connect_insecure()
            .unwrap();   
        let (_, sender) = client.split().unwrap();
        
        Self {
            client: Client::new(),
            socket_sender: sender,
            graphql_host: graphql_host,
            graphql_socket_host: graphql_socket_host,
            incremented_id: 0
        }
    }
    
    pub fn query(&self, query: String) -> ResponseStream {        
        let request = format!("{}?query={}", self.graphql_host, query);
        return ResponseStream::new(self.client.get(&request).send());
    }
    
    pub fn mutation(&self, query: String) -> ResponseStream {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let request = format!("{{\"query\":\"{}\"}}", query);       
        return ResponseStream::new(self.client.post(&self.graphql_host)
            .headers(headers)
            .body(request)
            .send());
    }
    
    pub fn subscribe(&mut self, query: String) -> SubscribeStream {
        self.incremented_id = self.incremented_id+1;
        let id = self.incremented_id;
                
        return SubscribeStream::new(id, query, &self.graphql_socket_host.clone());
    }
    
    pub fn unsubscribe(&mut self, id: u64) {
        SubscribeStream::unsubscribe(id, &mut self.socket_sender);
    }
}