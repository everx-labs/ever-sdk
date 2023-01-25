use crate::error::{ClientError, ClientResult};
use crate::net::{NetworkContext, ResultOfSubscription};
use async_trait::async_trait;
use std::future::Future;
use std::sync::Arc;
use ton_client_msg_mon::{
    EverApiProvider, EverApiSubscription, MessageMonitoringParams, MessageMonitoringResult,
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
                .map(|result| serde_json::from_value(result.result).unwrap())
                .map_err(|err| err.into());
            // We have to clone callback because it will move out of closure scope
            let callback = callback.clone();
            async move {
                callback(result).await;
            }
        };
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
