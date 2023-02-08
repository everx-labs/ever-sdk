/*
* Copyright 2018-2021 TON Labs LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate api_derive;

mod error;
mod message_monitor;
mod sdk_services;
#[cfg(test)]
mod tests;

pub use error::{Error, Result};
pub use message_monitor::{
    MessageMonitor, MessageMonitoringParams, MessageMonitoringResult, MessageMonitoringStatus,
    MessageMonitoringTransaction, MessageMonitoringTransactionCompute, MonitorFetchWait,
    MonitoredMessage, MonitoringQueueInfo,
};
pub use sdk_services::{MessageMonitorSdkServices, NetSubscription};
