/*
 * Copyright 2018-2021 TON Labs LTD.
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
use crate::error::{AddNetworkUrl, ClientError, ClientResult};
use crate::net::endpoint::Endpoint;
use crate::net::gql::{GraphQLMessageFromClient, GraphQLMessageFromServer};
use crate::net::server_link::NetworkState;
use crate::net::ton_gql::{GraphQLQuery, GraphQLQueryEvent};
use crate::net::{Error, NetworkConfig};
use futures::stream::{Fuse, FusedStream};
use futures::Sink;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;

type WSSender = Pin<Box<dyn Sink<String, Error = ClientError> + Send>>;

#[derive(Debug)]
enum HandlerAction {
    StartOperation(GraphQLQuery, Sender<GraphQLQueryEvent>),
    StopOperation(u32),

    Suspend,
    Resume,

    CheckKeepAlivePassed,
}

impl HandlerAction {
    async fn send(self, sender: &mut Sender<Self>) {
        if let Err(err) = sender.send(self).await {
            log::error!("HandlerAction.send failed {}", err);
        }
    }
}

//================================================================================== WebsocketLink

#[derive(Clone)]
pub(crate) struct WebsocketLink {
    handler_action_sender: Sender<HandlerAction>,
}

impl WebsocketLink {
    pub fn new(
        client_env: Arc<ClientEnv>,
        state: Arc<NetworkState>,
        config: NetworkConfig,
    ) -> Self {
        Self {
            handler_action_sender: LinkHandler::run(client_env, state, config),
        }
    }

    pub async fn start_operation(
        &self,
        operation: GraphQLQuery,
    ) -> ClientResult<Receiver<GraphQLQueryEvent>> {
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
    operation: GraphQLQuery,
    event_sender: Sender<GraphQLQueryEvent>,
}

impl RunningOperation {
    async fn notify(&mut self, event: GraphQLQueryEvent) {
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
    Suspended,
}

pub(crate) struct LinkHandler {
    client_env: Arc<ClientEnv>,
    action_receiver: Fuse<ReceiverStream<HandlerAction>>,
    internal_action_sender: Sender<HandlerAction>,
    internal_action_receiver: Fuse<ReceiverStream<HandlerAction>>,
    last_operation_id: u32,
    operations: HashMap<u32, RunningOperation>,
    keep_alive: KeepAlive,
    state: Arc<NetworkState>,
    config: NetworkConfig,
}

async fn ws_send(ws: &mut WSSender, message: GraphQLMessageFromClient) {
    log::debug!("Send WS message\n{}", message.get_message());
    let _ = ws.send(message.get_message()).await;
}

impl LinkHandler {
    fn run(
        client_env: Arc<ClientEnv>,
        state: Arc<NetworkState>,
        config: NetworkConfig,
    ) -> Sender<HandlerAction> {
        let (action_sender, action_receiver) = channel(10);
        let action_receiver = ReceiverStream::new(action_receiver);
        let (internal_action_sender, internal_action_receiver) = channel(10);
        let internal_action_receiver = ReceiverStream::new(internal_action_receiver);
        client_env.clone().spawn(Box::pin(async move {
            LinkHandler {
                client_env,
                action_receiver: action_receiver.fuse(),
                internal_action_sender,
                internal_action_receiver: internal_action_receiver.fuse(),
                last_operation_id: 0,
                operations: HashMap::new(),
                keep_alive: KeepAlive::WaitFirst,
                state,
                config,
            }
            .run_loop()
            .await;
        }));
        action_sender
    }

    async fn run_loop(&mut self) {
        let mut phase = Phase::Idle;
        while !self.action_receiver.is_terminated()
            && (phase == Phase::Idle || phase == Phase::Suspended)
        {
            let (internal_action, action) = futures::select!(
                internal_action = self.internal_action_receiver.select_next_some() => (Some(internal_action), None),
                action = self.action_receiver.select_next_some() => (None, Some(action)),
            );
            let suspended = phase == Phase::Suspended;
            if let Some(action) = internal_action.or(action) {
                phase = self.handle_idle_action(action, phase).await;
            }
            if phase == Phase::Connecting {
                phase = self.run_ws(suspended).await;
            }
        }
    }

    async fn run_ws(&mut self, suspended: bool) -> Phase {
        let ws = match self.connect().await {
            Ok(ws) => {
                if suspended {
                    self.send_error_to_running_operations(
                        Error::network_module_resumed()
                            .add_network_url_from_state(&self.state)
                            .await,
                    )
                    .await;
                }
                ws
            }
            Err(err) => {
                return self
                    .handle_network_error(Error::graphql_websocket_init_error(err), suspended)
                    .await;
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
        let _ = ws_sender
            .send(GraphQLMessageFromClient::ConnectionTerminate.get_message())
            .await;
        let _ = ws_sender.send(String::new());
        phase
    }

    async fn handle_idle_action(&mut self, action: HandlerAction, phase: Phase) -> Phase {
        match action {
            HandlerAction::StartOperation(payload, event_sender) => {
                self.start_operation(payload, event_sender, None, phase == Phase::Suspended)
                    .await;
                if phase == Phase::Suspended {
                    Phase::Suspended
                } else {
                    Phase::Connecting
                }
            }
            HandlerAction::StopOperation(id) => {
                let _ = self.operations.remove(&id);
                phase
            }
            HandlerAction::Suspend => Phase::Suspended,
            HandlerAction::Resume => Phase::Connecting,
            HandlerAction::CheckKeepAlivePassed => phase,
        }
    }

    async fn connect(&mut self) -> ClientResult<WebSocket> {
        self.keep_alive = KeepAlive::WaitFirst;
        let endpoint = self.state.get_query_endpoint().await?;
        let mut headers = HashMap::new();
        headers.insert("Sec-WebSocket-Protocol".into(), "graphql-ws".into());
        for (name, value) in Endpoint::http_headers(&self.config) {
            headers.insert(name, value);
        }
        let mut ws = self
            .client_env
            .websocket_connect(&endpoint.subscription_url, Some(headers))
            .await;
        if let Ok(ref mut ws) = ws {
            let mut connection_params = json!({});
            if let Some((name, value)) = &self.config.get_auth_header() {
                connection_params[name] = Value::String(value.clone());
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
                self.start_operation(operation, event_sender, ws, false)
                    .await;
            }
            HandlerAction::StopOperation(id) => {
                self.stop_operation(id, ws).await;
            }
            HandlerAction::Suspend => {
                if let Some(ws) = ws {
                    self.stop_running_operations(ws).await;
                    self.send_error_to_running_operations(Error::network_module_suspended())
                        .await;
                }
                next_phase = Phase::Suspended;
            }
            HandlerAction::Resume => {}
            HandlerAction::CheckKeepAlivePassed => match self.keep_alive {
                KeepAlive::WaitFirst => {}
                KeepAlive::WaitSecond { .. } => {}
                KeepAlive::WaitNext { .. } => {
                    self.keep_alive = KeepAlive::WaitFirst;
                    next_phase = self
                        .handle_network_error(
                            Error::websocket_disconnected("keep alive message wasn't received"),
                            false,
                        )
                        .await;
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
                log::debug!("Error received from websocket");
                return self
                    .handle_network_error(Error::websocket_disconnected(err), false)
                    .await;
            }
        };

        let mut next_phase = phase;
        match message {
            GraphQLMessageFromServer::ConnectionAck => {
                self.start_running_operations(ws).await;
                next_phase = Phase::Connected;
            }
            GraphQLMessageFromServer::ConnectionKeepAlive => {
                if let Some(phase) = self.check_latency().await {
                    next_phase = phase;
                } else {
                    match self.keep_alive {
                        KeepAlive::WaitFirst => {
                            self.keep_alive = KeepAlive::WaitSecond {
                                since_first_time: self.client_env.now_ms(),
                            };
                        }
                        KeepAlive::WaitSecond { since_first_time } => {
                            self.start_keep_alive_timer(
                                (self.client_env.now_ms() - since_first_time) * 2,
                            );
                        }
                        KeepAlive::WaitNext { timeout } => {
                            self.keep_alive = KeepAlive::Passed { timeout }
                        }
                        KeepAlive::Passed { .. } => {}
                    }
                }
            }
            GraphQLMessageFromServer::ConnectionError { error } => {
                next_phase = self
                    .handle_network_error(
                        Error::graphql_server_error(Some("connection"), &vec![error]),
                        false,
                    )
                    .await;
            }
            GraphQLMessageFromServer::Data { id, data, errors } => {
                let event = if let Some(errors) = errors {
                    GraphQLQueryEvent::Error(
                        Error::graphql_server_error(Some("operation"), &errors)
                            .add_network_url_from_state(&self.state)
                            .await,
                    )
                } else {
                    GraphQLQueryEvent::Data(data)
                };
                self.notify_with_remove(false, &id, event).await;
            }
            GraphQLMessageFromServer::Error { id, error } => {
                self.notify_with_remove(
                    true,
                    &id,
                    GraphQLQueryEvent::Error(Error::graphql_server_error(None, &[error])),
                )
                .await;
            }
            GraphQLMessageFromServer::Complete { id } => {
                self.notify_with_remove(true, &id, GraphQLQueryEvent::Complete)
                    .await;
            }
        }
        next_phase
    }

    async fn check_latency(&mut self) -> Option<Phase> {
        if !self.state.has_multiple_endpoints() {
            return None;
        }
        let current = self.state.query_endpoint().await?;
        if self.client_env.now_ms() < current.next_latency_detection_time() {
            return None;
        }
        let result = self.state.refresh_query_endpoint().await;
        match result {
            Ok(_) if current.latency() <= self.config.max_latency as u64 => None,
            Ok(_) => Some(
                self.handle_network_error(
                    Error::websocket_disconnected("Current endpoint has a critical sync latency."),
                    false,
                )
                .await,
            ),
            Err(err) => Some(self.handle_network_error(err, false).await),
        }
    }

    fn start_keep_alive_timer(&mut self, timeout: u64) {
        log::debug!("WS keep alive timer {}", timeout);
        let sender = self.internal_action_sender.clone();
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
        operation_event: GraphQLQueryEvent,
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

    async fn handle_network_error(&mut self, err: ClientError, suspended: bool) -> Phase {
        self.send_error_to_running_operations(err.add_network_url_from_state(&self.state).await)
            .await;
        if !suspended {
            self.send_error_to_running_operations(Error::network_module_suspended())
                .await;
        }
        self.state.internal_suspend().await;

        // send resume - it will try to reconnect after internal suspend timer in NetworkState ends
        HandlerAction::Resume
            .send(&mut self.internal_action_sender.clone())
            .await;
        // switch to Suspended phase
        Phase::Suspended
    }

    async fn send_error_to_running_operations(&mut self, err: ClientError) {
        for (_, operation) in &mut self.operations {
            operation
                .notify(GraphQLQueryEvent::Error(err.clone()))
                .await;
        }
    }

    async fn stop_running_operations(&self, ws: &mut WSSender) {
        for (id, _) in &self.operations {
            ws_send(ws, GraphQLMessageFromClient::Stop { id: id.to_string() }).await;
        }
    }

    async fn start_running_operations(&self, ws: &mut WSSender) {
        for (id, operation) in &self.operations {
            ws_send(ws, operation.operation.get_start_message(id.to_string())).await;
        }
    }

    async fn start_operation(
        &mut self,
        operation: GraphQLQuery,
        event_sender: Sender<GraphQLQueryEvent>,
        ws: Option<&mut WSSender>,
        suspended: bool,
    ) {
        let mut id = self.last_operation_id.wrapping_add(1);
        while id == 0 || self.operations.contains_key(&id) {
            id = id.wrapping_add(1);
        }

        let mut operation = RunningOperation {
            operation,
            event_sender,
        };

        operation.notify(GraphQLQueryEvent::Id(id)).await;
        if suspended {
            operation
                .notify(GraphQLQueryEvent::Error(Error::network_module_suspended()))
                .await;
        }

        if let Some(ws) = ws {
            ws_send(ws, operation.operation.get_start_message(id.to_string())).await;
        }

        self.operations.insert(id, operation);
        self.last_operation_id = id;
    }

    async fn stop_operation(&mut self, id: u32, ws: Option<&mut WSSender>) {
        if let Some(mut operation) = self.operations.remove(&id) {
            operation.notify(GraphQLQueryEvent::Complete).await;
            if let Some(ws) = ws {
                ws_send(ws, GraphQLMessageFromClient::Stop { id: id.to_string() }).await;
            }
        }
    }
}
