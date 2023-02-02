use crate::message_monitor::message::{MessageMonitoringParams, MessageMonitoringResult};
use crate::message_monitor::queue::MonitoringQueue;
use crate::sdk_services::{MessageMonitorSdkServices, NetSubscription};
use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, Mutex, RwLock};

/// The main message monitor object.
/// Incorporates and serves all message monitoring queues.
///
pub struct MessageMonitor<Sdk: MessageMonitorSdkServices> {
    /// External SDK services used by message monitor
    sdk: Sdk,
    /// Active queues
    queues: Arc<RwLock<HashMap<String, MonitoringQueue>>>,
    notify_queues: Arc<tokio::sync::watch::Sender<bool>>,
    listen_queues: tokio::sync::watch::Receiver<bool>,
    active_subscription: Mutex<Option<NetSubscription>>,
}

#[derive(Deserialize, Serialize, ApiType)]
pub struct MonitoringQueueInfo {
    /// Count of the unresolved messages.
    pub unresolved: u32,
    /// Count of resolved results.
    pub resolved: u32,
}

#[derive(Deserialize, Serialize, ApiType)]
pub enum MonitorFetchWait {
    /// If there are an unresolved messages and no resolved results yet,
    /// then monitor awaits for the next resolved result.
    /// If there are no unresolved messages then monitor immediately
    /// returns a resolved list (even if it is empty).
    AtLeastOne,

    /// Monitor waits until all unresolved messages will be resolved.
    /// If there are no unresolved messages then monitor immediately
    /// returns a resolved list (even if it is empty).
    All,

    // Monitor does not any awaits even if there are no resolved results yet.
    NoWait,
}

// pub
impl<SdkServices: MessageMonitorSdkServices> MessageMonitor<SdkServices> {
    pub fn new(sdk: SdkServices) -> Self {
        let (sender, receiver) = tokio::sync::watch::channel(false);
        Self {
            sdk,
            queues: Arc::new(RwLock::new(HashMap::new())),
            active_subscription: Mutex::new(None),
            notify_queues: Arc::new(sender),
            listen_queues: receiver,
        }
    }

    pub async fn monitor_messages(
        &self,
        queue: &str,
        messages: Vec<MessageMonitoringParams>,
    ) -> crate::error::Result<()> {
        {
            let mut queues = self.queues.write().unwrap();
            let queue = if let Some(queue) = queues.get_mut(queue) {
                queue
            } else {
                queues.insert(queue.to_string(), MonitoringQueue::new());
                queues.get_mut(queue).unwrap()
            };
            for message in messages {
                queue.add_unresolved(&self.sdk, message)?;
            }
            self.notify_queues.send(true).ok();
        }
        self.resubscribe().await?;
        Ok(())
    }

    pub async fn fetch_next_monitor_results(
        &self,
        queue: &str,
        wait: MonitorFetchWait,
    ) -> crate::error::Result<Vec<MessageMonitoringResult>> {
        loop {
            let results = {
                let mut queues = self.queues.write().unwrap();
                if let Some(queue) = queues.get_mut(queue) {
                    let is_ready = match wait {
                        MonitorFetchWait::NoWait => true,
                        MonitorFetchWait::AtLeastOne => !queue.resolved.is_empty(),
                        MonitorFetchWait::All => queue.unresolved.is_empty(),
                    };
                    if is_ready {
                        Some(queue.fetch_resolved())
                    } else {
                        None
                    }
                } else {
                    Some(vec![])
                }
            };
            if let Some(results) = results {
                if !results.is_empty() {
                    self.notify_queues.send(true).ok();
                }
                return Ok(results);
            }
            self.listen_queues.clone().changed().await.ok();
        }
    }

    pub fn get_queue_info(&self, queue: &str) -> crate::error::Result<MonitoringQueueInfo> {
        let queues = self.queues.read().unwrap();
        let (unresolved, resolved) = if let Some(queue) = queues.get(queue) {
            (queue.unresolved.len() as u32, queue.resolved.len() as u32)
        } else {
            (0, 0)
        };
        Ok(MonitoringQueueInfo {
            unresolved,
            resolved,
        })
    }

    pub fn cancel_monitor(&self, queue: &str) -> crate::error::Result<()> {
        let mut queues = self.queues.write().unwrap();
        queues.remove(queue);
        self.notify_queues.send(true).ok();
        Ok(())
    }
}

// priv
impl<SdkServices: MessageMonitorSdkServices> MessageMonitor<SdkServices> {
    async fn resubscribe(&self) -> crate::error::Result<()> {
        let new_subscription = self.subscribe().await?;
        let old_subscription = {
            mem::replace(
                &mut *self.active_subscription.lock().unwrap(),
                new_subscription,
            )
        };
        if let Some(old_subscription) = old_subscription {
            self.sdk.unsubscribe(old_subscription).await?;
        }
        Ok(())
    }

    async fn subscribe(&self) -> crate::error::Result<Option<NetSubscription>> {
        let messages = self.collect_unresolved();
        if messages.is_empty() {
            return Ok(None);
        }
        let queues = self.queues.clone();
        let notify_queues = self.notify_queues.clone();
        let callback = move |results| {
            if let Ok(results) = results {
                for queue in queues.write().unwrap().values_mut() {
                    queue.resolve(&results);
                }
            }
            notify_queues.send(true).ok();
            async {}
        };
        Ok(Some(
            self.sdk
                .subscribe_for_recent_ext_in_message_statuses(messages, callback)
                .await?,
        ))
    }

    fn collect_unresolved(&self) -> Vec<MessageMonitoringParams> {
        let mut messages = Vec::new();
        for queue in self.queues.read().unwrap().values() {
            for message in queue.unresolved.values() {
                messages.push(message.clone());
            }
        }
        messages
    }
}
