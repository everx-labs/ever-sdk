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

use futures::stream::{Stream, StreamExt};
use futures::sink::{Sink, SinkExt};
use serde_json::Value;
use tokio_tungstenite::tungstenite::{Message, Error as WsError};

#[derive(Debug, failure::Fail)]
pub enum GraphiteError {

    #[fail(display = "Network error: {}", 0)]
    NetworkError(String),

    #[fail(display = "Grpahql server returned error: {}", 0)]
    GraprhqlError(String),

    #[fail(display = "Server response parse error: {}\nresponse: {}", 0, 1)]
    SerdeError(serde_json::Error, String),

    #[fail(display = "{}", 0)]
    Other(String)
}

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
    read: std::pin::Pin<Box<dyn Stream<Item = Result<Value, GraphiteError>> + Send>>,
    write: Box<dyn Sink<Message, Error = WsError> + Unpin + Send>,
}

impl SubscribeStream {
    pub async fn new(id: u32, request: VariableRequest, host:&str) -> Result<Self, GraphiteError> {
        let (client, _) = tokio_tungstenite::connect_async(host)
            .await
            .map_err(|err| 
                GraphiteError::NetworkError(
                    format!("Can't create websocket client with address {}. Error {}", host, err)))?;

        let (mut write, read) = client.split();

        let query = request.get_query();
        let variables = request.get_variables();
        let request = if let Some(vars) = variables {
            format!(
                "{{\"id\":{}, \"type\": \"start\", \"payload\":{{ \"query\": \"{}\", \"variables\": {} }}}}",
                &id, &query, &vars)
        } else {
            format!(
                "{{\"id\":{}, \"type\": \"start\", \"payload\":{{ \"query\": \"{}\" }}}}",
                &id, &query)
        };

        let msg = Message::Text(request);
        write.send(msg)
            .await
            .map_err(|err| 
                GraphiteError::NetworkError(
                    format!("Sending message across stdin channel failed. Error: {}", err)))?;

        let read = read.filter_map(|result| async move {
            match result {
                Ok(message) => {
                    match message {
                        Message::Text(text) => {
                            match serde_json::from_str(text.as_str()) {
                                Ok(value) => {
                                    if let Some(error) = try_extract_error(&value) {
                                        return Some(Err(error));
                                    }
                                    Some(Ok(value))
                                }
                                Err(error) => {
                                    Some(Err(GraphiteError::SerdeError(error, text)))
                                }
                            }
                        },
                        _ => None
                    }
                },
                Err(err) => Some(Err(GraphiteError::NetworkError(
                    format!("Can not recieve next message: {}", err.to_string())))),
            }
        });

        Ok(Self {
            id: id,
            read: Box::pin(read),
            write: Box::new(write)
        })
    }

    pub async fn unsubscribe(&mut self) -> Result<(), GraphiteError> {
        let query = format!("{{\"id\":{}, \"type\": \"stop\", \"payload\":{{}}}}", &self.id);
        let msg = Message::Text(query.to_string());
        println!("Before unsubscribe");
        self.write.send(msg)
            .await
            .map_err(|err| 
                GraphiteError::NetworkError(
                    format!("Sending message across stdin channel failed. Error: {}", err)))?;
        println!("After unsubscribe");
        Ok(())
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
}

impl Stream for SubscribeStream {
    type Item = Result<Value, GraphiteError>;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut futures::task::Context<'_>) -> futures::task::Poll<Option<Self::Item>> {
        Stream::poll_next(self.read.as_mut(), cx)
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
                            return Some(GraphiteError::GraprhqlError(string.to_string()))
                        }
                    }
                }
            }
        }
    }

    return None;
}
