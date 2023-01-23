use crate::monitor::{MessageMonitoringParams, MessageMonitoringResult};
use crate::providers::{EverApiProvider, Subscription};
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
    ) -> crate::error::Result<Subscription> {
        let subscription = {
            let mut next = self.state.next_subscription.lock().unwrap();
            *next += 1;
            self.state.subscriptions.write().unwrap().insert(*next);
            *next
        };
        let state = self.state.clone();
        tokio::spawn(async move {
            let mut messages = messages;
            while !messages.is_empty() {
                let (found, not_found) = state.find_results(messages);
                messages = not_found;
                if !found.is_empty() {
                    {
                        if !state.subscriptions.read().unwrap().contains(&subscription) {
                            break;
                        }
                    }
                    callback(Ok(found)).await
                } else {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        });
        Ok(Subscription(subscription))
    }

    async fn unsubscribe(&self, subscription: Subscription) -> crate::error::Result<()> {
        self.state
            .subscriptions
            .write()
            .unwrap()
            .remove(&subscription.0);
        Ok(())
    }
}
