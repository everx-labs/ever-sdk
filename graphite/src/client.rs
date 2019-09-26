use crate::types::{ResponseStream, VariableRequest, SubscribeStream, GraphiteError};

use reqwest::Client as HttpClient;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

pub struct GqlClient {
    client_htpp: HttpClient,
    graphql_host: String,
    graphql_socket_host: String,
    incremented_id: u64
}

impl GqlClient {
    pub fn new(queries_server: &str, subscriptions_server: &str) -> Self {

        Self {
            client_htpp: HttpClient::new(),
            graphql_host: queries_server.to_owned(),
            graphql_socket_host: subscriptions_server.to_owned(),
            incremented_id: 0
        }
    }
    
    pub fn query(&self, query: String) -> Result<ResponseStream, GraphiteError> {        
        let request = format!("{}?query={}", self.graphql_host, query);
        //Ok(PeriodicRequestStream::new(self.client_htpp.get(&request))?)
        Ok(ResponseStream::new(self.client_htpp.get(&request).send())?)
    }

    pub fn query_vars(&self, request: VariableRequest) -> Result<ResponseStream, GraphiteError> {
        let request = match request.get_variables() {
            Some(vars) =>  format!("{{ \"query\": \"{}\", \"variables\": {} }}", request.get_query(), vars),
            None =>  format!("{{ \"query\": \"{}\" }}", request.get_query())
        };

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        Ok(ResponseStream::new(self.client_htpp.post(&self.graphql_host)
            .headers(headers)
            .body(request)
            .send())?)
    }
    
    pub fn mutation(&self, query: String) -> Result<ResponseStream, GraphiteError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let request = format!("{{\"query\":\"{}\"}}", query);       
        Ok(ResponseStream::new(self.client_htpp.post(&self.graphql_host)
            .headers(headers)
            .body(request)
            .send())?)
    }
    
    pub fn subscribe(&mut self, request: VariableRequest) -> Result<SubscribeStream, GraphiteError> {
        self.incremented_id = self.incremented_id+1;
        let id = self.incremented_id;
                
        Ok(SubscribeStream::new(id, request, &self.graphql_socket_host)?)
    }
}