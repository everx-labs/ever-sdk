use crate::message_monitor::{MessageMonitoringParams, MessageMonitoringResult};
use crate::{MessageMonitorSdkServices, MonitorFetchWaitMode};
use std::collections::HashMap;
use std::mem;

pub(crate) struct MonitoringQueue {
    pub unresolved: HashMap<String, MessageMonitoringParams>,
    pub resolved: Vec<MessageMonitoringResult>,
}

impl MonitoringQueue {
    pub fn add_unresolved<Sdk: MessageMonitorSdkServices>(
        &mut self,
        sdk: &Sdk,
        message: MessageMonitoringParams,
    ) -> crate::Result<()> {
        self.unresolved.insert(message.message.hash(sdk)?, message);
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
