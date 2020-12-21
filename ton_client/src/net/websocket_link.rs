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
 *
 */

use crate::client::{ClientEnv, WebSocket};
use crate::error::{ClientError, ClientResult};
use crate::net::gql::{
    GraphQLMessageFromClient, GraphQLMessageFromServer, GraphQLOperation, GraphQLOperationEvent,
};
use crate::net::server_info::ServerInfo;
use crate::net::{Error, NetworkConfig};
use futures::stream::{Fuse, FusedStream};
use futures::Sink;
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};

type WSSender = Pin<Box<dyn Sink<String, Error = ClientError> + Send>>;

#[derive(Debug)]
enum HandlerAction {
    StartOperation(GraphQLOperation, Sender<GraphQLOperationEvent>),
    StopOperation(u32),

    Suspend,
    Resume,

    CheckKeepAlivePassed,
}

impl HandlerAction {
    async fn send(self, sender: &mut Sender<Self>) {
        if let Err(err) = sender.send(self).await {
            println!("LinkAction.send failed {}", err);
        }
    }
}

//================================================================================== WebsocketLink

#[derive(Clone)]
pub(crate) struct WebsocketLink {
    handler_action_sender: Sender<HandlerAction>,
}

impl WebsocketLink {
    pub fn new(config: NetworkConfig, client_env: Arc<ClientEnv>) -> Self {
        Self {
            handler_action_sender: LinkHandler::run(config, client_env),
        }
    }

    pub async fn start_operation(
        &self,
        operation: GraphQLOperation,
    ) -> ClientResult<Receiver<GraphQLOperationEvent>> {
        let (event_sender, event_receiver) = channel(1);
        self.send_action_to_handler(HandlerAction::StartOperation(operation, event_sender))
            .await;
        Ok(event_receiver)
    }

    pub async fn stop_operation(&self, id: u32) {
        self.send_action_to_handler(HandlerAction::StopOperation(id))
            .await
    }

    pub async fn suspend(&self) {
        self.send_action_to_handler(HandlerAction::Suspend).await
    }

    pub async fn resume(&self) {
        self.send_action_to_handler(HandlerAction::Resume).await;
    }

    async fn send_action_to_handler(&self, action: HandlerAction) {
        action.send(&mut self.handler_action_sender.clone()).await
    }
}

#[derive(Clone)]
struct RunningOperation {
    operation: GraphQLOperation,
    event_sender: Sender<GraphQLOperationEvent>,
}

impl RunningOperation {
    async fn notify(&mut self, event: GraphQLOperationEvent) {
        let _ = self.event_sender.send(event).await;
    }
}

enum KeepAlive {
    WaitFirst,
    WaitSecond { since_first_time: u64 },
    WaitNext { timeout: u64 },
    Passed { timeout: u64 },
}

#[derive(PartialEq, Debug)]
enum Phase {
    Idle,
    Connecting,
    Connected,
}

pub(crate) struct LinkHandler {
    config: NetworkConfig,
    client_env: Arc<ClientEnv>,
    action_receiver: Fuse<Receiver<HandlerAction>>,
    internal_action_sender: Sender<HandlerAction>,
    internal_action_receiver: Fuse<Receiver<HandlerAction>>,
    last_operation_id: u32,
    operations: HashMap<u32, RunningOperation>,
    keep_alive: KeepAlive,
}

async fn ws_send(ws: &mut WSSender, message: GraphQLMessageFromClient) {
    let _ = ws.send(message.get_message()).await;
}

impl LinkHandler {
    fn run(config: NetworkConfig, client_env: Arc<ClientEnv>) -> Sender<HandlerAction> {
        let (action_sender, action_receiver) = channel(1);
        let (internal_action_sender, internal_action_receiver) = channel(1);
        client_env.clone().spawn(Box::pin(async move {
            LinkHandler {
                config,
                client_env,
                action_receiver: action_receiver.fuse(),
                internal_action_sender,
                internal_action_receiver: internal_action_receiver.fuse(),
                last_operation_id: 0,
                operations: HashMap::new(),
                keep_alive: KeepAlive::WaitFirst,
            }
            .run_loop()
            .await;
        }));
        action_sender
    }

    async fn run_loop(&mut self) {
        let mut phase = Phase::Idle;
        while !self.action_receiver.is_terminated() && phase == Phase::Idle {
            let (internal_action, action) = futures::select!(
                internal_action = self.internal_action_receiver.select_next_some() => (Some(internal_action), None),
                action = self.action_receiver.select_next_some() => (None, Some(action)),
            );
            if let Some(action) = internal_action.or(action) {
                phase = self.handle_idle_action(action).await;
            }
            if phase == Phase::Connecting {
                phase = self.run_ws().await;
            }
        }
    }

    async fn run_ws(&mut self) -> Phase {
        let ws = match self.connect().await {
            Ok(w) => w,
            Err(err) => {
                self.send_error_to_running_operations(err).await;
                return Phase::Idle;
            }
        };
        let mut phase = Phase::Connecting;
        let mut ws_receiver = ws.receiver.fuse();
        let mut ws_sender = ws.sender;
        while !self.action_receiver.is_terminated()
            && (phase == Phase::Connecting || phase == Phase::Connected)
        {
            let (message, internal_action, action) = futures::select!(
                message = ws_receiver.select_next_some() => (Some(message), None, None),
                internal_action = self.internal_action_receiver.select_next_some() => (None, Some(internal_action), None),
                action = self.action_receiver.select_next_some() => (None, None, Some(action)),
            );
            if let Some(message) = message {
                phase = self.handle_ws_message(message, &mut ws_sender, phase).await
            }
            if let Some(action) = internal_action.or(action) {
                phase = self.handle_ws_action(action, &mut ws_sender, phase).await
            }
        }
        phase
    }

    async fn handle_idle_action(&mut self, action: HandlerAction) -> Phase {
        match action {
            HandlerAction::StartOperation(payload, event_sender) => {
                self.start_operation(payload, event_sender, None).await;
                Phase::Connecting
            }
            HandlerAction::StopOperation(id) => {
                let _ = self.operations.remove(&id);
                Phase::Idle
            }
            HandlerAction::Suspend => Phase::Idle,
            HandlerAction::Resume => Phase::Connecting,
            HandlerAction::CheckKeepAlivePassed => Phase::Idle,
        }
    }

    async fn connect(&mut self) -> ClientResult<WebSocket> {
        self.keep_alive = KeepAlive::WaitFirst;
        let server_info = ServerInfo::fetch(
            self.client_env.clone(),
            &ServerInfo::expand_address(&self.config.server_address),
        )
        .await?;
        let address = server_info.subscription_url;
        let mut headers = HashMap::new();
        headers.insert("Sec-WebSocket-Protocol".into(), "graphql-ws".into());
        for (name, value) in ServerInfo::http_headers() {
            headers.insert(name, value);
        }
        let mut ws = self
            .client_env
            .websocket_connect(&address, Some(headers))
            .await;
        if let Ok(ref mut ws) = ws {
            let mut connection_params = json!({});
            if let Some(access_key) = &self.config.access_key {
                connection_params["accessKey"] = access_key.as_str().into();
            }
            let init_message = GraphQLMessageFromClient::ConnectionInit { connection_params };
            ws_send(&mut ws.sender, init_message).await;
        }
        ws
    }

    async fn handle_ws_action(
        &mut self,
        action: HandlerAction,
        ws: &mut WSSender,
        phase: Phase,
    ) -> Phase {
        let ws = if phase == Phase::Connected {
            Some(ws)
        } else {
            None
        };
        let mut next_phase = phase;
        match action {
            HandlerAction::StartOperation(operation, event_sender) => {
                self.start_operation(operation, event_sender, ws).await;
            }
            HandlerAction::StopOperation(id) => {
                self.stop_operation(id, ws).await;
            }
            HandlerAction::Suspend => {
                if let Some(ws) = ws {
                    self.stop_running_operations(ws).await;
                }
                next_phase = Phase::Idle
            }
            HandlerAction::Resume => {}
            HandlerAction::CheckKeepAlivePassed => match self.keep_alive {
                KeepAlive::WaitFirst => {}
                KeepAlive::WaitSecond { .. } => {}
                KeepAlive::WaitNext { .. } => {
                    self.keep_alive = KeepAlive::WaitFirst;
                    self.send_error_to_running_operations(Error::websocket_disconnected(
                        "keep alive message wasn't received",
                    ))
                    .await;
                    next_phase = Phase::Idle;
                }
                KeepAlive::Passed { timeout } => {
                    self.start_keep_alive_timer(timeout);
                }
            },
        }
        next_phase
    }

    async fn handle_ws_message(
        &mut self,
        message: ClientResult<String>,
        ws: &mut WSSender,
        phase: Phase,
    ) -> Phase {
        // Parse GraphQL message
        let message = match message {
            Ok(message) => match GraphQLMessageFromServer::parse(&message) {
                Ok(message) => message,
                _ => {
                    // Invalid message received, skip it and continue websocket loop
                    return phase;
                }
            },
            Err(err) => {
                self.send_error_to_running_operations(Error::websocket_disconnected(err))
                    .await;
                return Phase::Idle;
            }
        };

        let mut next_phase = phase;
        match message {
            GraphQLMessageFromServer::ConnectionAck => {
                self.start_running_operations(ws).await;
                next_phase = Phase::Connected;
            }
            GraphQLMessageFromServer::ConnectionKeepAlive => match self.keep_alive {
                KeepAlive::WaitFirst => {
                    self.keep_alive = KeepAlive::WaitSecond {
                        since_first_time: self.client_env.now_ms(),
                    };
                }
                KeepAlive::WaitSecond { since_first_time } => {
                    self.start_keep_alive_timer((self.client_env.now_ms() - since_first_time) * 2);
                }
                KeepAlive::WaitNext { timeout } => self.keep_alive = KeepAlive::Passed { timeout },
                KeepAlive::Passed { .. } => {}
            },
            GraphQLMessageFromServer::ConnectionError { error } => {
                self.send_error_to_running_operations(Error::graphql_server_error(
                    "connection",
                    &vec![error],
                ))
                .await;
                next_phase = Phase::Idle;
            }
            GraphQLMessageFromServer::Data { id, data, errors } => {
                let event = if let Some(errors) = errors {
                    GraphQLOperationEvent::Error(Error::graphql_server_error("operation", &errors))
                } else {
                    GraphQLOperationEvent::Data(data)
                };
                self.notify_with_remove(false, &id, event).await;
            }
            GraphQLMessageFromServer::Error { id, error } => {
                self.notify_with_remove(
                    true,
                    &id,
                    GraphQLOperationEvent::Error(Error::graphql_error(error)),
                )
                .await;
            }
            GraphQLMessageFromServer::Complete { id } => {
                self.notify_with_remove(true, &id, GraphQLOperationEvent::Complete)
                    .await;
            }
        }
        next_phase
    }

    fn start_keep_alive_timer(&mut self, timeout: u64) {
        let mut sender = self.internal_action_sender.clone();
        self.keep_alive = KeepAlive::WaitNext { timeout };
        let env = self.client_env.clone();
        env.clone().spawn(Box::pin(async move {
            let _ = env.set_timer(timeout).await;
            let _ = sender.send(HandlerAction::CheckKeepAlivePassed).await;
        }));
    }

    async fn notify_with_remove(
        &mut self,
        remove: bool,
        operation_id: &str,
        operation_event: GraphQLOperationEvent,
    ) {
        if let Ok(id) = u32::from_str_radix(operation_id, 10) {
            if remove {
                if let Some(mut operation) = self.operations.remove(&id) {
                    operation.notify(operation_event).await;
                }
            } else if let Some(operation) = self.operations.get_mut(&id) {
                operation.notify(operation_event).await;
            }
        }
    }

    async fn send_error_to_running_operations(&mut self, err: ClientError) {
        for (_, operation) in &mut self.operations {
            operation
                .notify(GraphQLOperationEvent::Error(err.clone()))
                .await;
        }
    }

    async fn stop_running_operations(&mut self, ws: &mut WSSender) {
        for (id, _) in self.operations.clone() {
            ws_send(ws, GraphQLMessageFromClient::Stop { id: id.to_string() }).await;
        }
    }

    async fn start_running_operations(&mut self, ws: &mut WSSender) {
        for (id, operation) in self.operations.clone() {
            ws_send(ws, operation.operation.get_start_message(id.to_string())).await;
        }
    }

    async fn start_operation(
        &mut self,
        operation: GraphQLOperation,
        event_sender: Sender<GraphQLOperationEvent>,
        ws: Option<&mut WSSender>,
    ) {
        let mut id = self.last_operation_id.wrapping_add(1);
        while id == 0 || self.operations.contains_key(&id) {
            id = id.wrapping_add(1);
        }

        let mut operation = RunningOperation {
            operation,
            event_sender,
        };

        operation.notify(GraphQLOperationEvent::Id(id)).await;

        if let Some(ws) = ws {
            ws_send(ws, operation.operation.get_start_message(id.to_string())).await;
        }

        self.operations.insert(id, operation);
        self.last_operation_id = id;
    }

    async fn stop_operation(&mut self, id: u32, ws: Option<&mut WSSender>) {
        if let Some(mut operation) = self.operations.remove(&id) {
            operation.notify(GraphQLOperationEvent::Complete).await;
            if let Some(ws) = ws {
                ws_send(ws, GraphQLMessageFromClient::Stop { id: id.to_string() }).await;
            }
        }
    }
}
