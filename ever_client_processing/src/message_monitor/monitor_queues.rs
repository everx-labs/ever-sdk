use crate::message_monitor::queue::{BufferedMessage, MonitoringQueue};
use crate::{MessageMonitoringParams, MonitoringQueueInfo};
use std::collections::{HashMap, HashSet};

pub(crate) const ADDING_TIMEOUT_MS: u64 = 1000;
const FETCHING_TIMEOUT_MS: u64 = 5000;

pub(crate) struct MonitorQueues {
    pub(crate) last_adding_time_ms: u64,
    pub(crate) last_fetching_time_ms: u64,
    pub(crate) queues: HashMap<String, MonitoringQueue>,
}

pub struct BufferedMessages {
    pub messages: Vec<MessageMonitoringParams>,
    pub hashes: HashSet<String>,
}

impl MonitorQueues {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
            last_adding_time_ms: 0,
            last_fetching_time_ms: 0,
        }
    }

    pub fn ensure(&mut self, name: &str) -> &mut MonitoringQueue {
        self.queues
            .entry(name.to_string())
            .or_insert_with(|| MonitoringQueue::new())
    }

    pub fn get_info(&self, queue: &str) -> MonitoringQueueInfo {
        self.queues
            .get(queue)
            .map(|x| x.get_info())
            .unwrap_or_default()
    }

    pub fn remove(&mut self, queue: &str) {
        self.queues.remove(queue);
    }

    pub fn add_buffered(&mut self, now_ms: u64, queue: &str, messages: Vec<BufferedMessage>) {
        if messages.is_empty() {
            return;
        }
        let queue = self.ensure(queue);
        for message in messages {
            queue.buffered.push(message);
        }
        self.last_adding_time_ms = now_ms;
    }

    pub fn has_buffered(&self) -> bool {
        self.queues
            .values()
            .find(|x| !x.buffered.is_empty())
            .is_some()
    }

    pub fn get_buffered(&self, now_ms: u64) -> Option<BufferedMessages> {
        let adding_timout_elapsed = now_ms > self.last_adding_time_ms + ADDING_TIMEOUT_MS;
        let fetching_timout_elapsed = now_ms > self.last_fetching_time_ms + FETCHING_TIMEOUT_MS;
        let is_time_to_resolve_buffered =
            (adding_timout_elapsed || fetching_timout_elapsed) && self.has_buffered();
        if !is_time_to_resolve_buffered {
            return None;
        }

        let mut buffered = BufferedMessages {
            messages: Vec::new(),
            hashes: HashSet::new(),
        };
        for queue in self.queues.values() {
            for message in &queue.buffered {
                if !buffered.hashes.contains(&message.hash) {
                    buffered.hashes.insert(message.hash.clone());
                    buffered.messages.push(message.message.clone());
                }
            }
        }
        if !buffered.messages.is_empty() {
            Some(buffered)
        } else {
            None
        }
    }

    pub fn start_resolving(&mut self, now_ms: u64, hashes: HashSet<String>) {
        self.last_fetching_time_ms = now_ms;
        for queue in self.queues.values_mut() {
            for i in (0..queue.buffered.len()).rev() {
                let hash = queue.buffered[i].hash.clone();
                if hashes.contains(&hash) {
                    let params = queue.buffered.remove(i);
                    queue.resolving.insert(hash, params.message.user_data);
                }
            }
        }
    }
}

impl MonitorQueues {}
