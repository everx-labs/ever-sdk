#[cfg(test)]
mod mock_sdk_services;

use crate::error;
use crate::message_monitor::{MessageMonitoringParams, MessageMonitoringResult};
#[cfg(test)]
pub use mock_sdk_services::MockSdkServices;
use std::future::Future;
use ton_types::Cell;

pub struct NetSubscription(pub usize);

#[async_trait]
pub trait MessageMonitorSdkServices {
    async fn subscribe_for_recent_ext_in_message_statuses<F: Future<Output = ()> + Send>(
        &self,
        messages: Vec<MessageMonitoringParams>,
        callback: impl Fn(error::Result<Vec<MessageMonitoringResult>>) -> F + Send + Sync + 'static,
    ) -> error::Result<NetSubscription>;

    async fn unsubscribe(&self, subscription: NetSubscription) -> error::Result<()>;

    fn cell_from_boc(&self, boc: &str, name: &str) -> error::Result<Cell>;
}
