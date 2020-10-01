/*
 * Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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

use crate::dispatch::DispatchTable;

#[cfg(test)]
mod tests;

mod blocks_walking;
mod errors;
mod fetching;
mod internal;
mod parsing;
mod process_message;
mod send_message;
mod types;
mod wait_for_transaction;

pub use errors::{Error, ErrorCode};
pub use process_message::{
    process_message, process_message_method, MessageSource, ParamsOfProcessMessage,
};
pub use send_message::{
    send_message, send_message_method, ParamsOfSendMessage, ResultOfSendMessage,
};
pub use types::{CallbackParams, ProcessingEvent, ProcessingState, TransactionOutput};
pub use wait_for_transaction::{
    wait_for_transaction, wait_for_transaction_method, ParamsOfWaitForTransaction,
    ResultOfWaitForTransaction,
};

use api_doc::reflect::TypeInfo;

pub const DEFAULT_NETWORK_RETRIES_LIMIT: i8 = -1;
pub const DEFAULT_NETWORK_RETRIES_TIMEOUT: u32 = 1000;
pub const DEFAULT_EXPIRATION_RETRIES_LIMIT: i8 = 20;
pub const DEFAULT_EXPIRATION_RETRIES_TIMEOUT: u32 = 1000;

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.register_api_types("processing", vec![CallbackParams::type_info]);
    handlers.spawn_method(send_message_method, send_message);
    handlers.spawn_method(wait_for_transaction_method, wait_for_transaction);
    handlers.spawn_method(process_message_method, process_message);
}
