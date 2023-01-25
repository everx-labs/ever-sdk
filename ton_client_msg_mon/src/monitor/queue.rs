use crate::monitor::{MessageMonitoringParams, MessageMonitoringResult};
use std::collections::HashMap;
use std::mem;
use ton_types::UInt256;

pub(crate) struct MonitoringQueue {
    pub unresolved: HashMap<UInt256, MessageMonitoringParams>,
    pub resolved: Vec<MessageMonitoringResult>,
}

impl MonitoringQueue {
    pub fn add_unresolved(&mut self, message: MessageMonitoringParams) {
        self.unresolved.insert(message.hash.clone(), message);
    }

    pub fn resolve(&mut self, results: &Vec<MessageMonitoringResult>) {
        for result in results {
            if self.unresolved.remove(&result.hash).is_some() {
                self.resolved.push(result.clone());
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

    pub fn fetch_resolved(&mut self) -> Vec<MessageMonitoringResult> {
        mem::take(&mut self.resolved)
    }
}
