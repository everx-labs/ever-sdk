use crate::message_monitor::message::{MessageMonitoringParams, MessageMonitoringResult};
use crate::message_monitor::monitor_queues::{BufferedMessages, MonitorQueues, ADDING_TIMEOUT_MS};
use crate::message_monitor::queue::BufferedMessage;
use crate::sdk_services::MessageMonitorSdkServices;
use crate::NetSubscription;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};

/// The main message monitor object.
/// Incorporates and serves all message monitoring queues.
///
pub struct MessageMonitor<Sdk: MessageMonitorSdkServices + Send + Sync + 'static> {
    state: Arc<MonitorState<Sdk>>,
}

struct MonitorState<Sdk: MessageMonitorSdkServices + Send + Sync + 'static> {
    /// External SDK services used by message monitor
    sdk: Sdk,

    /// Active queues
    queues: RwLock<MonitorQueues>,

    notify_resolved: Arc<tokio::sync::watch::Sender<crate::error::Result<()>>>,
    listen_resolved: tokio::sync::watch::Receiver<crate::error::Result<()>>,
    active_subscriptions: Mutex<HashMap<usize, HashSet<String>>>,
}

#[derive(Deserialize, Serialize, ApiType, Default)]
pub struct MonitoringQueueInfo {
    /// Count of the unresolved messages.
    pub unresolved: u32,
    /// Count of resolved results.
    pub resolved: u32,
}

#[derive(Deserialize, Serialize, ApiType, Copy, Clone)]
pub enum MonitorFetchWaitMode {
    /// If there are no resolved results yet, then monitor awaits for the next resolved result.
    AtLeastOne,

    /// Monitor waits until all unresolved messages will be resolved.
    /// If there are no unresolved messages then monitor will wait.
    All,

    // Monitor does not any awaits even if there are no resolved results yet.
    NoWait,
}

impl<SdkServices: MessageMonitorSdkServices + Send + Sync> MessageMonitor<SdkServices> {
    pub fn new(sdk: SdkServices) -> Self {
        Self {
            state: Arc::new(MonitorState::new(sdk)),
        }
    }

    pub fn monitor_messages(
        &self,
        queue: &str,
        messages: Vec<MessageMonitoringParams>,
    ) -> crate::error::Result<()> {
        self.state.monitor_messages(queue, messages)
    }

    pub async fn fetch_next_monitor_results(
        &self,
        queue: &str,
        wait_mode: MonitorFetchWaitMode,
    ) -> crate::error::Result<Vec<MessageMonitoringResult>> {
        self.state
            .fetch_next_monitor_results(queue, wait_mode)
            .await
    }

    pub fn get_queue_info(&self, queue: &str) -> crate::error::Result<MonitoringQueueInfo> {
        self.state.get_queue_info(queue)
    }

    pub fn cancel_monitor(&self, queue: &str) -> crate::error::Result<()> {
        self.state.cancel_monitor(queue)
    }
}

impl<Sdk: MessageMonitorSdkServices + Send + Sync> MonitorState<Sdk> {
    fn new(sdk: Sdk) -> Self {
        let (sender, receiver) = tokio::sync::watch::channel(Ok(()));
        Self {
            sdk,
            queues: RwLock::new(MonitorQueues::new()),
            active_subscriptions: Mutex::new(HashMap::new()),
            notify_resolved: Arc::new(sender),
            listen_resolved: receiver,
        }
    }

    fn monitor_messages(
        self: &Arc<Self>,
        queue: &str,
        messages: Vec<MessageMonitoringParams>,
    ) -> crate::error::Result<()> {
        if messages.is_empty() {
            return Ok(());
        }
        let mut buffered = Vec::new();
        for message in messages {
            buffered.push(BufferedMessage {
                hash: message.message.hash(&self.sdk)?,
                message,
            });
        }

        let mut queues = self.queues.write().unwrap();
        let should_start_buffering_timer = !queues.has_buffered();
        let now_ms = self.sdk.now_ms();
        queues.add_buffered(now_ms, queue, buffered);
        if should_start_buffering_timer {
            queues.last_fetching_time_ms = now_ms;
            self.clone().start_buffering_timer();
        }
        Ok(())
    }

    async fn fetch_next_monitor_results(
        &self,
        queue: &str,
        wait_mode: MonitorFetchWaitMode,
    ) -> crate::error::Result<Vec<MessageMonitoringResult>> {
        let mut listen_resolved = self.listen_resolved.clone();
        loop {
            if let Some(fetched) = self.fetch_next(queue, wait_mode) {
                return Ok(fetched);
            }
            listen_resolved.changed().await.unwrap();
            if let Err(err) = listen_resolved.borrow().as_ref() {
                return Err(err.clone());
            }
        }
    }

    fn get_queue_info(&self, queue: &str) -> crate::error::Result<MonitoringQueueInfo> {
        Ok(self.queues.read().unwrap().get_info(queue))
    }

    fn cancel_monitor(&self, queue: &str) -> crate::error::Result<()> {
        self.queues.write().unwrap().remove(queue);
        Ok(())
    }

    fn start_buffering_timer(self: Arc<Self>) {
        self.clone().sdk.spawn(async move {
            loop {
                let _ = self.sdk.sleep(ADDING_TIMEOUT_MS).await;
                let now_ms = self.sdk.now_ms();
                let buffered = self.queues.read().unwrap().get_buffered(now_ms);
                if let Some(buffered) = buffered {
                    let hashes = buffered.hashes.clone();
                    if self.clone().subscribe(buffered).await.is_ok() {
                        let mut queues = self.queues.write().unwrap();
                        queues.start_resolving(now_ms, hashes);
                        if !queues.has_buffered() {
                            break;
                        }
                    }
                }
            }
        });
    }

    async fn subscribe(self: Arc<Self>, buffered: BufferedMessages) -> crate::error::Result<()> {
        if buffered.messages.is_empty() {
            return Ok(());
        }
        let self1 = self.clone();
        let callback = move |results| {
            let self1 = self1.clone();
            async move {
                match results {
                    Ok(results) => {
                        let empty_subscriptions =
                            self1.resolve_results_and_return_empty_subscriptions(&results);
                        for subscription in empty_subscriptions {
                            let _ = self1.sdk.unsubscribe(subscription).await;
                        }
                        self1.notify_resolved.send(Ok(())).ok();
                    }
                    Err(err) => {
                        self1.notify_resolved.send(Err(err)).ok();
                    }
                }
            }
        };
        let subscription = self
            .sdk
            .subscribe_for_recent_ext_in_message_statuses(buffered.messages, callback)
            .await?;
        self.active_subscriptions
            .lock()
            .unwrap()
            .insert(subscription.0, buffered.hashes);
        Ok(())
    }

    fn resolve_results_and_return_empty_subscriptions(
        &self,
        results: &Vec<MessageMonitoringResult>,
    ) -> Vec<NetSubscription> {
        let mut queues = self.queues.write().unwrap();
        for queue in queues.queues.values_mut() {
            queue.resolve(&results);
        }

        let mut active_subscriptions = self.active_subscriptions.lock().unwrap();
        let empty_subscriptions = Self::resolve_subscriptions(&mut active_subscriptions, results);

        for subscription in &empty_subscriptions {
            active_subscriptions.remove(&subscription.0);
        }
        empty_subscriptions
    }

    fn resolve_subscriptions(
        active_subscriptions: &mut HashMap<usize, HashSet<String>>,
        results: &Vec<MessageMonitoringResult>,
    ) -> Vec<NetSubscription> {
        let mut empty_subscriptions = HashSet::new();
        for (subscription, hashes) in &mut *active_subscriptions {
            for result in results {
                if hashes.remove(&result.hash) {
                    if hashes.is_empty() {
                        empty_subscriptions.insert(*subscription);
                    }
                }
            }
        }
        empty_subscriptions
            .into_iter()
            .map(|x| NetSubscription(x))
            .collect()
    }

    fn fetch_next(
        &self,
        queue: &str,
        wait_mode: MonitorFetchWaitMode,
    ) -> Option<Vec<MessageMonitoringResult>> {
        let mut queues = self.queues.write().unwrap();
        let (fetched, queue_should_be_removed) = if let Some(queue) = queues.queues.get_mut(queue) {
            let next = queue.fetch_next(wait_mode);
            let should_be_removed =
                queue.results.is_empty() && queue.resolving.is_empty() && queue.buffered.is_empty();
            (next, should_be_removed)
        } else if let MonitorFetchWaitMode::NoWait = wait_mode {
            (Some(vec![]), false)
        } else {
            (None, false)
        };
        if queue_should_be_removed {
            queues.remove(queue);
        }
        fetched
    }
}
