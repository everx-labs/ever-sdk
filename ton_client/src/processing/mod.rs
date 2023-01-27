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
 *
 */

#[cfg(test)]
mod tests;

pub(crate) mod blocks_walking;
mod errors;
mod fetching;
mod internal;
mod message_monitor;
mod message_monitor_providers;
pub(crate) mod parsing;
pub(crate) mod process_message;
mod remp;
pub(crate) mod send_message;
mod send_messages;
mod types;
pub(crate) mod wait_for_transaction;

pub use errors::{Error, ErrorCode};
pub use message_monitor::{
    cancel_monitor, cancel_monitor_api, fetch_next_monitor_results, fetch_next_monitor_results_api,
    get_monitor_info, get_monitor_info_api, monitor_messages, monitor_messages_api,
    ParamsOfCancelMonitor, ParamsOfFetchNextMonitorResults, ParamsOfGetMonitorInfo,
    ParamsOfMonitorMessages, ResultOfFetchNextMonitorResults,
};
pub(crate) use message_monitor_providers::MessageMonitorEverApi;
pub use process_message::{process_message, ParamsOfProcessMessage};
pub use send_message::{send_message, ParamsOfSendMessage, ResultOfSendMessage};
pub use send_messages::{
    send_messages, send_messages_api, ParamsOfSendMessages, ResultOfSendMessages,
};
pub use ton_client_processing::{
    MessageMonitoringParams, MessageMonitoringResult, MessageMonitoringTransaction,
    MessageMonitoringTransactionCompute, MonitorFetchWait, MonitoredMessage, MonitoringQueueInfo,
};
pub use types::{DecodedOutput, ProcessingEvent, ProcessingResponseType, ResultOfProcessMessage};
pub use wait_for_transaction::{wait_for_transaction, ParamsOfWaitForTransaction};
