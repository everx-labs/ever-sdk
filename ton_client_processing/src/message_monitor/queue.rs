use crate::message_monitor::MessageMonitoringResult;
use crate::MonitorFetchWaitMode;
use serde_json::Value;
use std::collections::HashMap;
use std::mem;

pub(crate) struct MonitoringQueue {
    pub unresolved: HashMap<String, Option<Value>>,
    pub resolved: Vec<MessageMonitoringResult>,
}

impl MonitoringQueue {
    pub fn add_unresolved(&mut self, hash: String, user_data: Option<Value>) {
        self.unresolved.insert(hash, user_data);
    }

    pub fn resolve(&mut self, results: &Vec<MessageMonitoringResult>) {
        for result in results {
            if let Some(user_data) = self.unresolved.remove(&result.hash) {
                let mut result = result.clone();
                result.user_data = user_data;
                self.resolved.push(result);
            }
        }
    }
}

impl MonitoringQueue {
    pub fn new() -> Self {
        Self {
            unresolved: HashMap::new(),
            resolved: Vec::new(),
        }
    }

    pub fn fetch_next(
        &mut self,
        wait_mode: MonitorFetchWaitMode,
    ) -> Option<Vec<MessageMonitoringResult>> {
        let is_ready = match wait_mode {
            MonitorFetchWaitMode::NoWait => true,
            MonitorFetchWaitMode::AtLeastOne => !self.resolved.is_empty(),
            MonitorFetchWaitMode::All => self.unresolved.is_empty() && !self.resolved.is_empty(),
        };
        if is_ready {
            Some(mem::take(&mut self.resolved))
        } else {
            None
        }
    }
}
