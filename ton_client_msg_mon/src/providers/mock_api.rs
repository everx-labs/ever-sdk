use crate::monitor::{MessageMonitoringParams, MessageMonitoringResult};
use crate::providers::{EverApiProvider, EverApiSubscription};
use std::collections::HashSet;
use std::future::Future;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

#[derive(Clone)]
pub struct MockEverApi {
    state: Arc<State>,
}

struct State {
    recent_ext_in_messages: RwLock<Vec<MessageMonitoringResult>>,
    next_subscription: Mutex<usize>,
    subscriptions: RwLock<HashSet<usize>>,
}

impl State {
    fn subscribe<F: Future<Output = ()> + Send>(
        self: Arc<Self>,
        messages: Vec<MessageMonitoringParams>,
        callback: impl Fn(crate::error::Result<Vec<MessageMonitoringResult>>) -> F
            + Send
            + Sync
            + 'static,
    ) -> usize {
        let subscription = self.create_subscription();
        tokio::spawn(async move {
            let mut messages = messages;
            while !messages.is_empty() && self.contains_subscription(subscription) {
                let (found, not_found) = self.find_results(messages);
                messages = not_found;
                if !found.is_empty() {
                    callback(Ok(found)).await
                } else {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        });
        subscription
    }

    fn find_results(
        &self,
        messages: Vec<MessageMonitoringParams>,
    ) -> (Vec<MessageMonitoringResult>, Vec<MessageMonitoringParams>) {
        let recent = self.recent_ext_in_messages.read().unwrap();
        let mut found = Vec::new();
        let mut not_found = Vec::new();
        for message in messages {
            if let Some(result) = recent.iter().find(|&x| x.hash == message.hash) {
                found.push(result.clone());
            } else {
                not_found.push(message);
            }
        }
        (found, not_found)
    }

    fn create_subscription(&self) -> usize {
        let mut next = self.next_subscription.lock().unwrap();
        *next += 1;
        self.subscriptions.write().unwrap().insert(*next);
        *next
    }

    fn contains_subscription(&self, subscription: usize) -> bool {
        self.subscriptions.read().unwrap().contains(&subscription)
    }

    fn remove_subscription(&self, subscription: usize) {
        self.subscriptions.write().unwrap().remove(&subscription);
    }
}

impl MockEverApi {
    pub fn new() -> Self {
        Self {
            state: Arc::new(State {
                recent_ext_in_messages: RwLock::new(Vec::new()),
                next_subscription: Mutex::new(1),
                subscriptions: RwLock::new(HashSet::new()),
            }),
        }
    }

    pub fn add_recent_ext_in_messages(&self, messages: Vec<MessageMonitoringResult>) {
        let mut recent = self.state.recent_ext_in_messages.write().unwrap();
        recent.extend(messages)
    }
}

#[async_trait]
impl EverApiProvider for MockEverApi {
    async fn subscribe_for_recent_ext_in_message_statuses<F: Future<Output = ()> + Send>(
        &self,
        messages: Vec<MessageMonitoringParams>,
        callback: impl Fn(crate::error::Result<Vec<MessageMonitoringResult>>) -> F
            + Send
            + Sync
            + 'static,
    ) -> crate::error::Result<EverApiSubscription> {
        Ok(EverApiSubscription(self.state.clone().subscribe(messages, callback)))
    }

    async fn unsubscribe(&self, subscription: EverApiSubscription) -> crate::error::Result<()> {
        self.state.remove_subscription(subscription.0);
        Ok(())
    }
}
