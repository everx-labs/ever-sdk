use crate::boc::cache::Bocs;
use crate::error::{ClientError, ClientResult};
use crate::net::{NetworkContext, ResultOfSubscription};
use async_trait::async_trait;
use serde_json::Value;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use ton_client_processing::{
    MessageMonitorSdkServices, MessageMonitoringParams, MessageMonitoringResult,
    MessageMonitoringStatus, MessageMonitoringTransaction, MessageMonitoringTransactionCompute,
    MonitoredMessage, NetSubscription,
};
use ever_block::Cell;

pub(crate) struct SdkServices {
    net: Arc<NetworkContext>,
    bocs: Arc<Bocs>,
}

impl SdkServices {
    pub fn new(net: Arc<NetworkContext>, bocs: Arc<Bocs>) -> Self {
        Self { net, bocs }
    }

    fn subscription(messages: Vec<MessageMonitoringParams>) -> (String, Option<Value>) {
        let query = r#"
        subscription monitorMessages($messages: [MessageMonitoringParams!]!) {
            recentExtInMessageStatuses(messages: $messages) {
                hash
                status
                error
                transaction {
                    hash
                    aborted
                    compute {
                        exit_code
                    }
                }
            }
        }
        "#;
        let messages = messages
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<GraphQLMessageMonitoringParams>>();
        (query.to_string(), Some(json!({ "messages": messages })))
    }
}

impl From<ClientError> for ton_client_processing::Error {
    fn from(value: ClientError) -> Self {
        Self {
            code: value.code,
            message: value.message,
            data: value.data,
        }
    }
}

impl From<ton_client_processing::Error> for ClientError {
    fn from(value: ton_client_processing::Error) -> Self {
        Self {
            code: value.code,
            message: value.message,
            data: value.data,
        }
    }
}

fn deserialize_subscription_data(
    value: ResultOfSubscription,
) -> ton_client_processing::Result<Vec<MessageMonitoringResult>> {
    let result = value.result;
    if result.is_null() {
        return Ok(vec![]);
    }
    let statuses = result.get("recentExtInMessageStatuses").ok_or_else(|| {
        crate::net::Error::invalid_server_response("missing required `recentExtInMessageStatuses`")
    })?;
    let result = serde_json::from_value::<GraphQLMessageMonitoringResult>(statuses.clone())
        .map_err(|err| crate::net::Error::invalid_server_response(err))?
        .into();
    Ok(vec![result])
}

#[async_trait]
impl MessageMonitorSdkServices for SdkServices {
    async fn subscribe_for_recent_ext_in_message_statuses<F: Future<Output = ()> + Send>(
        &self,
        messages: Vec<MessageMonitoringParams>,
        callback: impl Fn(ton_client_processing::Result<Vec<MessageMonitoringResult>>) -> F
            + Send
            + Sync
            + 'static,
    ) -> ton_client_processing::Result<NetSubscription> {
        // We have to wrap callback into Arc because it will move out of closure scope
        let callback = Arc::new(callback);
        let (query, vars) = Self::subscription(messages);
        let retry_start = Arc::new(AtomicU64::new(0));
        let net_state = self.net.get_server_link()?.state();
        let subscription = self
            .net
            .subscribe(
                query,
                vars,
                move |evt: ClientResult<ResultOfSubscription>| {
                    // We have to clone callback because it will move out of closure scope
                    let callback = callback.clone();
                    let net_state = net_state.clone();
                    let retry_start = retry_start.clone();
                    async move {
                        match evt {
                            Ok(evt) => {
                                retry_start.store(0, Ordering::Relaxed);
                                callback(deserialize_subscription_data(evt)).await;
                            }
                            Err(err) => {
                                let mut start = retry_start.load(Ordering::Relaxed);
                                if start == 0 {
                                    start = net_state.env().now_ms();
                                    retry_start.store(start, Ordering::Relaxed);
                                }
                                if err.code == crate::net::ErrorCode::NetworkModuleSuspended as u32
                                    || err.code
                                        == crate::net::ErrorCode::NetworkModuleResumed as u32
                                {
                                    return;
                                }
                                if !crate::client::Error::is_network_error(&err)
                                    || !net_state.can_retry_network_error(start)
                                {
                                    callback(Err(err.into()));
                                }
                            }
                        }
                    }
                },
            )
            .await?;
        Ok(NetSubscription(subscription as usize))
    }

    async fn unsubscribe(
        &self,
        subscription: NetSubscription,
    ) -> ton_client_processing::Result<()> {
        Ok(self.net.unsubscribe(subscription.0 as u32).await?)
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.net.env.spawn(future);
    }

    async fn sleep(&self, ms: u64) -> ton_client_processing::Result<()> {
        self.net.env.set_timer(ms).await?;
        Ok(())
    }

    fn now_ms(&self) -> u64 {
        self.net.env.now_ms()
    }

    fn cell_from_boc(&self, boc: &str, name: &str) -> ton_client_processing::Result<Cell> {
        let (_, cell) = self.bocs.deserialize_cell(boc, name)?;
        Ok(cell)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphQLMessageMonitoringParams {
    pub hash: Option<String>,
    pub address: Option<String>,
    pub boc: Option<String>,
    pub wait_until: u32,
}

impl From<MessageMonitoringParams> for GraphQLMessageMonitoringParams {
    fn from(value: MessageMonitoringParams) -> Self {
        match value.message {
            MonitoredMessage::Boc { boc } => Self {
                address: None,
                hash: None,
                boc: Some(boc),
                wait_until: value.wait_until,
            },
            MonitoredMessage::HashAddress { hash, address } => Self {
                address: Some(address),
                hash: Some(hash),
                boc: None,
                wait_until: value.wait_until,
            },
        }
    }
}

#[derive(Deserialize)]
struct GraphQLMessageMonitoringResult {
    pub hash: String,
    pub status: GraphQLMessageMonitoringStatus,
    pub transaction: Option<GraphQLMessageMonitoringTransaction>,
    pub error: Option<String>,
}

impl From<GraphQLMessageMonitoringResult> for MessageMonitoringResult {
    fn from(value: GraphQLMessageMonitoringResult) -> Self {
        Self {
            hash: value.hash,
            status: value.status.into(),
            transaction: value.transaction.map(|x| x.into()),
            error: value.error,
            user_data: None,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GraphQLMessageMonitoringStatus {
    Finalized,
    Timeout,
    Reserved,
}

impl From<GraphQLMessageMonitoringStatus> for MessageMonitoringStatus {
    fn from(value: GraphQLMessageMonitoringStatus) -> Self {
        match value {
            GraphQLMessageMonitoringStatus::Finalized => Self::Finalized,
            GraphQLMessageMonitoringStatus::Timeout => Self::Timeout,
            GraphQLMessageMonitoringStatus::Reserved => Self::Reserved,
        }
    }
}

#[derive(Deserialize)]
struct GraphQLMessageMonitoringTransaction {
    pub hash: Option<String>,
    pub aborted: bool,
    pub compute: Option<GraphQLMessageMonitoringTransactionCompute>,
}

impl From<GraphQLMessageMonitoringTransaction> for MessageMonitoringTransaction {
    fn from(value: GraphQLMessageMonitoringTransaction) -> Self {
        Self {
            hash: value.hash,
            aborted: value.aborted,
            compute: value.compute.map(|x| x.into()),
        }
    }
}

#[derive(Deserialize)]
struct GraphQLMessageMonitoringTransactionCompute {
    pub exit_code: i32,
}

impl From<GraphQLMessageMonitoringTransactionCompute> for MessageMonitoringTransactionCompute {
    fn from(value: GraphQLMessageMonitoringTransactionCompute) -> Self {
        Self {
            exit_code: value.exit_code,
        }
    }
}
