use crate::message_monitor::MessageMonitoringResult;
use crate::{MessageMonitoringParams, MonitorFetchWaitMode, MonitoringQueueInfo};
use serde_json::Value;
use std::collections::HashMap;
use std::mem;

#[derive(Clone)]
pub(crate) struct BufferedMessage {
    pub message: MessageMonitoringParams,
    pub hash: String,
}

pub(crate) struct MonitoringQueue {
    pub buffered: Vec<BufferedMessage>,
    pub resolving: HashMap<String, Option<Value>>,
    pub results: Vec<MessageMonitoringResult>,
}

impl MonitoringQueue {
    pub fn resolve(&mut self, results: &Vec<MessageMonitoringResult>) {
        for result in results {
            if let Some(user_data) = self.resolving.remove(&result.hash) {
                let mut result = result.clone();
                result.user_data = user_data;
                self.results.push(result);
            }
        }
    }

    pub fn get_info(&self) -> MonitoringQueueInfo {
        MonitoringQueueInfo {
            unresolved: (self.buffered.len() + self.resolving.len()) as u32,
            resolved: self.results.len() as u32,
        }
    }
}

impl MonitoringQueue {
    pub fn new() -> Self {
        Self {
            buffered: Vec::new(),
            resolving: HashMap::new(),
            results: Vec::new(),
        }
    }

    pub fn fetch_next(
        &mut self,
        wait_mode: MonitorFetchWaitMode,
    ) -> Option<Vec<MessageMonitoringResult>> {
        let is_ready = match wait_mode {
            MonitorFetchWaitMode::NoWait => true,
            MonitorFetchWaitMode::AtLeastOne => !self.results.is_empty(),
            MonitorFetchWaitMode::All => {
                self.buffered.is_empty()
                    && self.resolving.is_empty()
                    && !self.results.is_empty()
            }
        };
        if is_ready {
            Some(mem::take(&mut self.results))
        } else {
            None
        }
    }
}
