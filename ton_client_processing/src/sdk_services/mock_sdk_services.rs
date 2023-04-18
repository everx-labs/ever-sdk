use crate::message_monitor::{CellFromBoc, MessageMonitoringParams, MessageMonitoringResult};
use crate::{error, Error, MessageMonitorSdkServices, NetSubscription};
use base64::Engine;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use ton_types::{Cell};

#[derive(Clone)]
pub struct MockSdkServices {
    state: Arc<State>,
}

struct State {
    results: RwLock<HashMap<String, MessageMonitoringResult>>,
    next_subscription: Mutex<usize>,
    subscriptions: RwLock<HashSet<usize>>,
}

impl CellFromBoc for State {
    fn convert(&self, boc: &str, name: &str) -> crate::Result<Cell> {
        State::cell_from_boc(boc, name)
    }
}

impl State {
    fn subscribe<F: Future<Output = ()> + Send>(
        self: Arc<Self>,
        messages: Vec<MessageMonitoringParams>,
        callback: impl Fn(error::Result<Vec<MessageMonitoringResult>>) -> F + Send + Sync + 'static,
    ) -> usize {
        let subscription = self.create_subscription();
        tokio::spawn(async move {
            let mut messages = messages
                .into_iter()
                .map(|x| (x.message.hash(&*self).unwrap(), x))
                .collect::<HashMap<_, _>>();
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
        messages: HashMap<String, MessageMonitoringParams>,
    ) -> (
        Vec<MessageMonitoringResult>,
        HashMap<String, MessageMonitoringParams>,
    ) {
        let recent = self.results.read().unwrap();
        let mut found = Vec::new();
        let mut not_found = HashMap::new();
        for (hash, message) in messages {
            if let Some(result) = recent.get(&hash) {
                found.push(result.clone());
            } else {
                not_found.insert(hash.clone(), message);
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

    fn cell_from_boc(boc: &str, name: &str) -> error::Result<Cell> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(boc)
            .map_err(|err| {
                Error::invalid_boc(format!("error decode {} BOC base64: {}", name, err))
            })?;
        ton_types::boc::read_single_root_boc(&bytes).map_err(|err| {
            Error::invalid_boc(format!("{} BOC deserialization error: {}", name, err))
        })
    }
}

impl MockSdkServices {
    pub fn new() -> Self {
        Self {
            state: Arc::new(State {
                results: RwLock::new(HashMap::new()),
                next_subscription: Mutex::new(1),
                subscriptions: RwLock::new(HashSet::new()),
            }),
        }
    }

    pub fn add_recent_ext_in_messages(&self, messages: Vec<MessageMonitoringResult>) {
        let mut recent = self.state.results.write().unwrap();
        recent.extend(messages.into_iter().map(|x| (x.hash.clone(), x)))
    }
}

#[async_trait]
impl MessageMonitorSdkServices for MockSdkServices {
    async fn subscribe_for_recent_ext_in_message_statuses<F: Future<Output = ()> + Send>(
        &self,
        messages: Vec<MessageMonitoringParams>,
        callback: impl Fn(error::Result<Vec<MessageMonitoringResult>>) -> F + Send + Sync + 'static,
    ) -> error::Result<NetSubscription> {
        Ok(NetSubscription(
            self.state.clone().subscribe(messages, callback),
        ))
    }

    async fn unsubscribe(&self, subscription: NetSubscription) -> error::Result<()> {
        self.state.remove_subscription(subscription.0);
        Ok(())
    }

    fn cell_from_boc(&self, boc: &str, name: &str) -> error::Result<Cell> {
        State::cell_from_boc(boc, name)
    }
}
