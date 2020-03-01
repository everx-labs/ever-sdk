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
    
    pub fn subscribe(&mut self, request: VariableRequest) -> Result<SubscribeStream, GraphiteError> {
        self.incremented_id = self.incremented_id+1;
        let id = self.incremented_id;
                
        Ok(SubscribeStream::new(id, request, &self.graphql_socket_host)?)
    }
}