use crate::error::{ClientError, ClientResult};
use crate::net::{NetworkContext, ResultOfSubscription};
use async_trait::async_trait;
use std::future::Future;
use std::sync::Arc;
use ton_client_msg_mon::{
    EverApiProvider, EverApiSubscription, MessageMonitoringParams, MessageMonitoringResult,
    MessageMonitoringStatus, MessageMonitoringTransaction, MessageMonitoringTransactionCompute,
};

pub(crate) struct MessageMonitorEverApi {
    net: Arc<NetworkContext>,
}

impl MessageMonitorEverApi {
    pub fn new(net: Arc<NetworkContext>) -> Self {
        Self { net }
    }
}

impl From<ClientError> for ton_client_msg_mon::Error {
    fn from(value: ClientError) -> Self {
        Self {
            code: value.code,
            message: value.message,
            data: value.data,
        }
    }
}

impl From<ton_client_msg_mon::Error> for ClientError {
    fn from(value: ton_client_msg_mon::Error) -> Self {
        Self {
            code: value.code,
            message: value.message,
            data: value.data,
        }
    }
}

#[async_trait]
impl EverApiProvider for MessageMonitorEverApi {
    async fn subscribe_for_recent_ext_in_message_statuses<F: Future<Output = ()> + Send>(
        &self,
        messages: Vec<MessageMonitoringParams>,
        callback: impl Fn(ton_client_msg_mon::Result<Vec<MessageMonitoringResult>>) -> F
            + Send
            + Sync
            + 'static,
    ) -> ton_client_msg_mon::Result<EverApiSubscription> {
        // We have to wrap callback into Arc because it will move out of closure scope
        let callback = Arc::new(callback);
        let subscription_callback = move |evt: ClientResult<ResultOfSubscription>| {
            let result = evt
                .map(|result| {
                    serde_json::from_value::<Vec<GraphQLMessageMonitoringResult>>(result.result)
                        .unwrap()
                        .into_iter()
                        .map(|x| x.into())
                        .collect::<Vec<_>>()
                })
                .map_err(|err| err.into());
            // We have to clone callback because it will move out of closure scope
            let callback = callback.clone();
            async move {
                callback(result).await;
            }
        };
        let messages = messages
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<GraphQLMessageMonitoringParams>>();
        let subscription = self
            .net
            .subscribe(
                r#"
                subscription monitorMessages($messages: [RecentExtInMsg!]!) {
                    recentExtInMsgStatuses(messages: $messages) {
                        hash
                        status
                        error
                        transaction {
                            hash
                            compute {
                                exit_code
                            }
                        }
                    }
                }
                "#
                .to_string(),
                Some(json!({ "messages": messages })),
                subscription_callback,
            )
            .await?;
        Ok(EverApiSubscription(subscription as usize))
    }

    async fn unsubscribe(
        &self,
        subscription: EverApiSubscription,
    ) -> ton_client_msg_mon::Result<()> {
        Ok(self.net.unsubscribe(subscription.0 as u32).await?)
    }
}

#[derive(Serialize)]
struct GraphQLMessageMonitoringParams {
    pub hash: String,
    pub address: String,
    pub wait_until: u32,
}

impl From<MessageMonitoringParams> for GraphQLMessageMonitoringParams {
    fn from(value: MessageMonitoringParams) -> Self {
        Self {
            hash: value.hash,
            address: value.address,
            wait_until: value.wait_until,
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
    RejectedByFullNode,
    IncludedIntoBlock,
}

impl From<GraphQLMessageMonitoringStatus> for MessageMonitoringStatus {
    fn from(value: GraphQLMessageMonitoringStatus) -> Self {
        match value {
            GraphQLMessageMonitoringStatus::Finalized => Self::Finalized,
            GraphQLMessageMonitoringStatus::Timeout => Self::Timeout,
            GraphQLMessageMonitoringStatus::RejectedByFullNode => Self::RejectedByFullNode,
            GraphQLMessageMonitoringStatus::IncludedIntoBlock => Self::IncludedIntoBlock,
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
