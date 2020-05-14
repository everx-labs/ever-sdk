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

use crate::types::{VariableRequest, SubscribeStream, GraphiteError};

use reqwest::{Client as HttpClient, ClientBuilder, Response};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

#[derive(Clone)]
pub struct GqlClient {
    client_htpp: HttpClient,
    graphql_host: String,
    graphql_socket_host: String,
    incremented_id: u64
}

impl GqlClient {
    pub fn new(queries_server: &str, subscriptions_server: &str) -> Result<Self, GraphiteError> {
        let client = ClientBuilder::new()  
            .build()
            .map_err(|err| GraphiteError::new(err.to_string()))?;

        Ok(Self {
            client_htpp: client,
            graphql_host: queries_server.to_owned(),
            graphql_socket_host: subscriptions_server.to_owned(),
            incremented_id: 0
        })
    }

    async fn process_response(response: Response) -> Result<serde_json::Value, GraphiteError> {
        match response.text().await {
            Ok(res_str) => {
                if let Ok(value) = serde_json::from_str(res_str.as_str()) {
                    if let Some(error) = crate::types::try_extract_error(&value) {
                        return Err(error);
                    }
                    Ok(value)
                } else {
                    Err(GraphiteError::new(format!(
                        "Invalid JSON: {}", res_str)))
                }
            },
            Err(err) => Err(GraphiteError::new(err.to_string().clone()))
        }
    }
    
    pub async fn query(&self, query: String) -> Result<serde_json::Value, GraphiteError> {
        let request = format!("{}?query={}", self.graphql_host, query);
        let response = self.client_htpp.get(&request)
            .send()
            .await
            .map_err(|err| 
                GraphiteError::new(format!("Can't send request: {}", err)))?;
        
        Self::process_response(response).await
    }

    pub async fn query_vars(&self, request: VariableRequest) -> Result<serde_json::Value, GraphiteError> {
        let request = match request.get_variables() {
            Some(vars) =>  format!("{{ \"query\": \"{}\", \"variables\": {} }}", request.get_query(), vars),
            None =>  format!("{{ \"query\": \"{}\" }}", request.get_query())
        };

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let response = self.client_htpp.post(&self.graphql_host)
            .headers(headers)
            .body(request)
            .send()
            .await
            .map_err(|err| 
                GraphiteError::new(format!("Can't send request: {}", err)))?;

        Self::process_response(response).await
    }
    
    pub fn subscribe(&self, request: VariableRequest) -> Result<SubscribeStream, GraphiteError> {
        Ok(SubscribeStream::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| GraphiteError::new(format!("Cannot get time: {}", err)))?
                .subsec_nanos(),
            request,
            &self.graphql_socket_host)?)
    }
}