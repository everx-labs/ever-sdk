use crate::message_monitor::{MessageMonitoringParams, MessageMonitoringResult};
use std::collections::HashMap;
use std::mem;

pub(crate) struct MonitoringQueue {
    pub unresolved: HashMap<String, MessageMonitoringParams>,
    pub resolved: Vec<MessageMonitoringResult>,
}

impl MonitoringQueue {
    pub fn add_unresolved(&mut self, message: MessageMonitoringParams) -> crate::Result<()> {
        self.unresolved.insert(message.message.hash()?, message);
        Ok(())
    }

    pub fn resolve(&mut self, results: &Vec<MessageMonitoringResult>) {
        for result in results {
            if let Some(params) = self.unresolved.remove(&result.hash) {
                let mut result = result.clone();
                result.user_data = params.user_data;
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

    pub fn fetch_resolved(&mut self) -> Vec<MessageMonitoringResult> {
        mem::take(&mut self.resolved)
    }
}
