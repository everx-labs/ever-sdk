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

extern crate futures;
extern crate websocket;

use futures::task::{Poll, Context};
use futures::stream::{Stream, StreamExt};
use futures::compat::{Compat01As03, Compat01As03Sink};
//use futures_util::sink::{Sink, SinkExt};
use serde_json::Value;
use websocket::{ClientBuilder, OwnedMessage};
use websocket::futures::Sink;
//use websocket::futures::Future;

use websocket::futures::Stream as WsStream;


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
    write: Option<Box<dyn websocket::futures::Sink<SinkItem = websocket::OwnedMessage, SinkError = websocket::WebSocketError> + Unpin + Send>>,
}

impl SubscribeStream {
    pub async fn new(id: u32, request: VariableRequest, host:&str) -> Result<Self, GraphiteError> {
        let client = ClientBuilder::new(host)
            .map_err(|err| 
                GraphiteError::NetworkError(
                    format!("Can't create websocket client with address {}. Error {}", host, err)))?
            .add_protocol("graphql-ws")
            .async_connect(None);

        let (client, _) = Compat01As03::new(client)
            .await
            .map_err(|err|
                GraphiteError::NetworkError(
                    format!("Can't connect to websocket server {}. Error {}", host, err)))?;


        let query = request.get_query();
        let variables = request.get_variables();
        let request: String;

        if let Some(vars) = variables {
            request = format!("{{\"id\":{}, \"type\": \"start\", \"payload\":{{ \"query\": \"{}\", \"variables\": {} }}}}", &id, &query, &vars);
        } else {
            request = format!("{{\"id\":{}, \"type\": \"start\", \"payload\":{{ \"query\": \"{}\" }}}}", &id, &query);
        }
        
        let (write, read) = client.split();
        let read = Compat01As03::new(read);

        let msg = OwnedMessage::Text(request);
        let write = Compat01As03::new(write.send(msg))
            .await
            .map_err(|err| 
                GraphiteError::NetworkError(
                    format!("Sending message across stdin channel failed. Error: {}", err)))?;

        let read = read.map(|result| {
            match result {
                Ok(message) => {
                    match message {
                        OwnedMessage::Text(text) => {
                            match serde_json::from_str(text.as_str()) {
                                Ok(value) => {
                                    if let Some(error) = try_extract_error(&value) {
                                        return Err(error);
                                    }
                                    Ok(value)
                                }
                                Err(error) => {
                                    Err(GraphiteError::SerdeError(error, text))
                                }
                            }
                        },
                        _ => Ok(Value::Null)
                    }
                },
                Err(err) => Err(GraphiteError::NetworkError(
                    format!("Can not recieve next message: {}", err.to_string()))),
            }
        });

        Ok(Self {
            id: id,
            read: Box::pin(read),
            write: Some(Box::new(write))
        })
    }

    // pub async fn subscribe(&mut self) -> Result<(), GraphiteError> {
    //     let query = &self.request.get_query().clone();
    //     let variables = &self.request.get_variables().clone();
    //     let request: String;

    //     if let Some(vars) = variables {
    //         request = format!("{{\"id\":{}, \"type\": \"start\", \"payload\":{{ \"query\": \"{}\", \"variables\": {} }}}}", &self.id, &query, &vars);
    //     } else {
    //         request = format!("{{\"id\":{}, \"type\": \"start\", \"payload\":{{ \"query\": \"{}\" }}}}", &self.id, &query);
    //     }

    //     let msg = OwnedMessage::Text(request);
    //     let send = self.sender.send(msg)
    //         .map_err(|err| 
    //             GraphiteError::NetworkError(
    //                 format!("Sending message across stdin channel failed. Error: {}", err)));

    //     Compat01As03::new(send).await?;

    //     Ok(())
    // }

    pub fn unsubscribe(&mut self) -> Result<(), GraphiteError> {
        let query = format!("{{\"id\":{}, \"type\": \"stop\", \"payload\":{{}}}}", &self.id);
        let msg = OwnedMessage::Text(query.to_string());
        let write = self.write.take().ok_or(
            GraphiteError::Other("Already unsubscribed".to_owned())
        )?;
        println!("Before unsubscribe");
        write.wait().send(msg)
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
                            return Some(GraphiteError::GraprhqlError(string.to_string()))
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

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Stream::poll_next(self.read.as_mut(), cx)
    }
}
