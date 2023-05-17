use crate::message_monitor::message::{MessageMonitoringParams, MessageMonitoringResult};
use crate::message_monitor::queue::MonitoringQueue;
use crate::sdk_services::MessageMonitorSdkServices;
use crate::NetSubscription;
use std::collections::{HashMap, HashSet};
use std::mem;
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
    buffering: Mutex<Buffering>,
    queues: RwLock<HashMap<String, MonitoringQueue>>,
    notify_resolved: Arc<tokio::sync::watch::Sender<crate::error::Result<()>>>,
    listen_resolved: tokio::sync::watch::Receiver<crate::error::Result<()>>,
    active_subscriptions: Mutex<HashMap<usize, HashSet<String>>>,
}

struct Buffering {
    messages: HashMap<String, Vec<MessageMonitoringParams>>,
    last_adding_time_ms: u64,
    last_fetching_time_ms: u64,
}

const ADDING_TIMEOUT_MS: u64 = 1000;
const FETCHING_TIMEOUT_MS: u64 = 5000;

#[derive(Deserialize, Serialize, ApiType)]
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

// pub
impl<SdkServices: MessageMonitorSdkServices + Send + Sync> MessageMonitor<SdkServices> {
    pub fn new(sdk: SdkServices) -> Self {
        Self {
            state: Arc::new(MonitorState::new(sdk)),
        }
    }

    pub async fn monitor_messages(
        &self,
        queue: &str,
        messages: Vec<MessageMonitoringParams>,
    ) -> crate::error::Result<()> {
        self.state.monitor_messages(queue, messages).await
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
            queues: RwLock::new(HashMap::new()),
            active_subscriptions: Mutex::new(HashMap::new()),
            notify_resolved: Arc::new(sender),
            listen_resolved: receiver,
            buffering: Mutex::new(Buffering {
                messages: HashMap::new(),
                last_adding_time_ms: 0,
                last_fetching_time_ms: 0,
            }),
        }
    }

    async fn monitor_messages(
        self: &Arc<Self>,
        queue: &str,
        messages: Vec<MessageMonitoringParams>,
    ) -> crate::error::Result<()> {
        let should_start_buffering_timer = self.add_to_buffering(queue, messages);
        if should_start_buffering_timer {
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
        let queues = self.queues.read().unwrap();
        let (mut unresolved, resolved) = if let Some(queue) = queues.get(queue) {
            (queue.unresolved.len() as u32, queue.resolved.len() as u32)
        } else {
            (0, 0)
        };
        let buffering = self.buffering.lock().unwrap();
        if let Some(queue) = buffering.messages.get(queue) {
            unresolved += queue.len() as u32;
        }
        Ok(MonitoringQueueInfo {
            unresolved,
            resolved,
        })
    }

    fn cancel_monitor(&self, queue: &str) -> crate::error::Result<()> {
        let mut queues = self.queues.write().unwrap();
        queues.remove(queue);
        let mut buffering = self.buffering.lock().unwrap();
        buffering.messages.remove(queue);
        Ok(())
    }

    fn add_to_buffering(&self, queue: &str, messages: Vec<MessageMonitoringParams>) -> bool {
        if messages.is_empty() {
            return false;
        }
        let mut buffering = self.buffering.lock().unwrap();
        let should_start_buffering_timer = buffering.messages.is_empty();
        let buffer = if let Some(buffer) = buffering.messages.get_mut(queue) {
            buffer
        } else {
            buffering.messages.insert(queue.to_string(), Vec::new());
            buffering.messages.get_mut(queue).unwrap()
        };
        for message in messages {
            buffer.push(message);
        }
        buffering.last_adding_time_ms = self.sdk.now_ms();
        should_start_buffering_timer
    }

    fn start_buffering_timer(self: Arc<Self>) {
        self.clone().sdk.spawn(async move {
            loop {
                let _ = self.sdk.sleep(ADDING_TIMEOUT_MS).await;
                let now_ms = self.sdk.now_ms();
                let messages = {
                    let mut buffering = self.buffering.lock().unwrap();
                    let is_adding_timout =
                        now_ms > buffering.last_adding_time_ms + ADDING_TIMEOUT_MS;
                    let is_fetching_timout =
                        now_ms > buffering.last_fetching_time_ms + FETCHING_TIMEOUT_MS;
                    if (is_adding_timout || is_fetching_timout) && !buffering.messages.is_empty() {
                        buffering.last_fetching_time_ms = now_ms;
                        Some(mem::replace(&mut buffering.messages, HashMap::new()))
                    } else {
                        None
                    }
                };
                if let Some(messages) = messages {
                    let _ = self.start_monitoring(messages).await;
                    break;
                }
            }
        });
    }

    async fn start_monitoring(
        self: &Arc<Self>,
        messages: HashMap<String, Vec<MessageMonitoringParams>>,
    ) -> crate::error::Result<()> {
        let (messages, hashes) = {
            let mut queues = self.queues.write().unwrap();
            let mut message_hashes = HashSet::new();
            let mut message_params = Vec::new();
            for (queue, messages) in messages {
                let queue = if let Some(queue) = queues.get_mut(&queue) {
                    queue
                } else {
                    queues.insert(queue.clone(), MonitoringQueue::new());
                    queues.get_mut(&queue).unwrap()
                };
                for message in messages {
                    let hash = message.message.hash(&self.sdk)?;
                    if !message_hashes.contains(&hash) {
                        queue.add_unresolved(hash.clone(), message.user_data.clone());
                        message_hashes.insert(hash.clone());
                        message_params.push(message);
                    }
                }
            }
            (message_params, message_hashes)
        };
        self.clone().subscribe(messages, hashes).await?;
        Ok(())
    }

    async fn subscribe(
        self: Arc<Self>,
        messages: Vec<MessageMonitoringParams>,
        hashes: HashSet<String>,
    ) -> crate::error::Result<()> {
        if messages.is_empty() {
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
            .subscribe_for_recent_ext_in_message_statuses(messages, callback)
            .await?;
        self.active_subscriptions
            .lock()
            .unwrap()
            .insert(subscription.0, hashes);
        Ok(())
    }

    fn resolve_results_and_return_empty_subscriptions(
        &self,
        results: &Vec<MessageMonitoringResult>,
    ) -> Vec<NetSubscription> {
        let mut queues = self.queues.write().unwrap();
        for queue in queues.values_mut() {
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
        let (fetched, queue_should_be_removed) = if let Some(queue) = queues.get_mut(queue) {
            let next = queue.fetch_next(wait_mode);
            let should_be_removed = queue.resolved.is_empty() && queue.unresolved.is_empty();
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
