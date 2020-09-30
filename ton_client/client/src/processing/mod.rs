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
mod defaults;
mod errors;
mod internal;
mod process_message;
mod send_message;
mod types;
mod wait_for_transaction;

pub use errors::{Error, ErrorCode};
pub use process_message::{MessageSource, ParamsOfProcessMessage, ResultOfProcessMessage};
pub use send_message::{
    send_message, send_message_method, ParamsOfSendMessage, ResultOfSendMessage,
};
pub use types::{CallbackParams, ProcessingEvent, ProcessingOptions, ProcessingState};
pub use wait_for_transaction::{
    wait_for_transaction, wait_for_transaction_method, ParamsOfWaitForTransaction,
    ResultOfWaitForTransaction,
};

use api_doc::reflect::TypeInfo;

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.register_api_types("processing", vec![CallbackParams::type_info]);
    handlers.spawn_method(send_message_method, send_message);
    handlers.spawn_method(wait_for_transaction_method, wait_for_transaction);
}
