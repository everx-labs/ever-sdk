use crate::monitor::message::{MessageMonitoringParams, MessageMonitoringResult};
use crate::monitor::queue::MonitoringQueue;
use crate::providers::{EverApiProvider, Subscription};
use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, Mutex, RwLock};

/// The main message monitor object.
/// Incorporates and serves all message monitoring queues.
///
pub struct MessageMonitor<EverApi: EverApiProvider> {
    /// External provider for Ever API
    api: EverApi,
    /// Active queues
    queues: Arc<RwLock<HashMap<String, MonitoringQueue>>>,
    ///
    notify_queues: Arc<tokio::sync::watch::Sender<bool>>,
    listen_queues: tokio::sync::watch::Receiver<bool>,
    active_subscription: Mutex<Option<Subscription>>,
}

pub struct MonitoringQueueInfo {
    pub queued: usize,
    pub resolved: usize,
}

pub enum MonitorFetchWait {
    /*
     * If there are no resolved results yet, then monitor awaits for next resolved result.
     * Note that if there are no queued messages and resolved buffer is empty
     * then monitor immediately returns an empty list.
     */
    AtLeastOne,
    /*
     * Monitor waits until all queued messages will be resolved.
     * Note that if there are no queued messages and resolved buffer is empty
     * then monitor immediately returns an empty list.
     */
    AllQueued,
    /*
     * Monitor does not any awaits even if there are no resolved results yet.
     */
    NoWait,
}

// pub
impl<EverApi: EverApiProvider> MessageMonitor<EverApi> {
    pub fn new(api: EverApi) -> Self {
        let (sender, receiver) = tokio::sync::watch::channel(false);
        Self {
            api,
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
            for message in messages {
                let queue = if let Some(queue) = queues.get_mut(queue) {
                    queue
                } else {
                    queues.insert(queue.to_string(), MonitoringQueue::new());
                    queues.get_mut(queue).unwrap()
                };
                queue.unresolved.push(message);
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
                        MonitorFetchWait::AllQueued => queue.unresolved.is_empty(),
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

    pub fn get_monitor_info(&self, queue: &str) -> crate::error::Result<MonitoringQueueInfo> {
        let queues = self.queues.read().unwrap();
        let (queued, resolved) = if let Some(queue) = queues.get(queue) {
            (queue.unresolved.len(), queue.resolved.len())
        } else {
            (0, 0)
        };
        Ok(MonitoringQueueInfo { queued, resolved })
    }
}

// priv
impl<EverApi: EverApiProvider> MessageMonitor<EverApi> {
    pub fn cancel_monitor(&self, queue: &str) -> crate::error::Result<()> {
        let mut queues = self.queues.write().unwrap();
        queues.remove(queue);
        self.notify_queues.send(true).ok();
        Ok(())
    }

    async fn resubscribe(&self) -> crate::error::Result<()> {
        let new_subscription = self.subscribe().await?;
        if let Some(old_subscription) = mem::replace(
            &mut *self.active_subscription.lock().unwrap(),
            new_subscription,
        ) {
            self.api.unsubscribe(old_subscription).await?;
        }
        Ok(())
    }

    async fn subscribe(&self) -> crate::error::Result<Option<Subscription>> {
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
            self.api
                .subscribe_for_recent_ext_in_message_statuses(messages, callback)
                .await?,
        ))
    }

    fn collect_unresolved(&self) -> Vec<MessageMonitoringParams> {
        let mut messages = Vec::new();
        for queue in self.queues.read().unwrap().values() {
            for message in &queue.unresolved {
                messages.push(message.clone());
            }
        }
        messages
    }
}
