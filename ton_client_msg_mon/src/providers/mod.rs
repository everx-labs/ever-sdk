#[cfg(test)]
mod mock_api;

use crate::error;
use crate::monitor::{MessageMonitoringParams, MessageMonitoringResult};
#[cfg(test)]
pub use mock_api::MockEverApi;
use std::future::Future;

pub struct Subscription(usize);

#[async_trait]
pub trait EverApiProvider {
    async fn subscribe_for_recent_ext_in_message_statuses<F: Future<Output = ()> + Send>(
        &self,
        messages: Vec<MessageMonitoringParams>,
        callback: impl Fn(error::Result<Vec<MessageMonitoringResult>>) -> F + Send + Sync + 'static,
    ) -> error::Result<Subscription>;

    async fn unsubscribe(&self, subscription: Subscription) -> error::Result<()>;
}
