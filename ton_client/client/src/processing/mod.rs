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
pub use process_message::{process_message, MessageSource, ParamsOfProcessMessage};
pub use send_message::{send_message, ParamsOfSendMessage, ResultOfSendMessage};
pub use types::{AbiDecodedOutput, CallbackParams, ProcessingEvent, TransactionOutput};
pub use wait_for_transaction::{wait_for_transaction, ParamsOfWaitForTransaction};

use api_doc::reflect::TypeInfo;

pub const DEFAULT_NETWORK_RETRIES_LIMIT: i8 = -1;
pub const DEFAULT_NETWORK_RETRIES_TIMEOUT: u32 = 1000;
pub const DEFAULT_EXPIRATION_RETRIES_LIMIT: i8 = 20;
pub const DEFAULT_EXPIRATION_RETRIES_TIMEOUT: u32 = 1000;

/// Message processing module.
///
/// This module incorporates functions related to complex message
/// processing scenarios.
#[derive(TypeInfo)]
#[type_info(name = "processing")]
pub struct ProcessingModule;

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.register_module::<ProcessingModule>(
        &[
            CallbackParams::type_info,
            MessageSource::type_info,
            ProcessingEvent::type_info,
            TransactionOutput::type_info,
            AbiDecodedOutput::type_info,
        ],
        |reg| {
            reg.async_func(send_message, send_message::send_message_info);
            reg.async_func(
                wait_for_transaction,
                wait_for_transaction::wait_for_transaction_info,
            );
            reg.async_func(process_message, process_message::process_message_info);
        },
    );
}
