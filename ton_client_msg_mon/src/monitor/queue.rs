use crate::monitor::{MessageMonitoringParams, MessageMonitoringResult};
use std::mem;

pub(crate) struct MonitoringQueue {
    pub unresolved: Vec<MessageMonitoringParams>,
    pub resolved: Vec<MessageMonitoringResult>,
}

impl MonitoringQueue {
    pub(crate) fn resolve(&mut self, results: &Vec<MessageMonitoringResult>) {
        for i in (0..self.unresolved.len()).rev() {
            if let Some(result) = results.iter().find(|&x| x.hash == self.unresolved[i].hash) {
                self.resolved.push(result.clone());
                self.unresolved.remove(i);
            }
        }
    }
}

impl MonitoringQueue {
    pub fn new() -> Self {
        Self {
            unresolved: Vec::new(),
            resolved: Vec::new(),
        }
    }

    pub fn fetch_resolved(&mut self) -> Vec<MessageMonitoringResult> {
        mem::replace(&mut self.resolved, Vec::new())
    }
}
