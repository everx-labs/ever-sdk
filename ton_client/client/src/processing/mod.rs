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

use crate::dispatch::{ModuleReg, Registrar};

#[cfg(test)]
mod tests;

mod blocks_walking;
mod errors;
mod fetching;
mod internal;
pub(crate) mod parsing;
pub(crate) mod process_message;
mod send_message;
mod types;
mod wait_for_transaction;

pub use errors::{Error, ErrorCode};
pub use process_message::{process_message, MessageSource, ParamsOfProcessMessage};
pub use send_message::{send_message, ParamsOfSendMessage, ResultOfSendMessage};
pub use types::{DecodedOutput, CallbackParams, ProcessingEvent, ResultOfProcessMessage};
pub use wait_for_transaction::{wait_for_transaction, ParamsOfWaitForTransaction};

pub const DEFAULT_NETWORK_RETRIES_LIMIT: i8 = -1;
pub const DEFAULT_NETWORK_RETRIES_TIMEOUT: u32 = 1000;
pub const DEFAULT_EXPIRATION_RETRIES_LIMIT: i8 = 20;
pub const DEFAULT_EXPIRATION_RETRIES_TIMEOUT: u32 = 1000;

/// Message processing module.
///
/// This module incorporates functions related to complex message
/// processing scenarios.
#[derive(ApiModule)]
#[api_module(name = "processing")]
pub struct ProcessingModule;

impl ModuleReg for ProcessingModule {
    fn reg(reg: &mut Registrar) {
        reg.t::<MessageSource>();
        reg.t::<ProcessingEvent>();
        reg.t::<ResultOfProcessMessage>();
        reg.t::<DecodedOutput>();

        reg.async_f_callback(send_message, send_message::send_message_api);
        reg.async_f_callback(
            wait_for_transaction,
            wait_for_transaction::wait_for_transaction_api,
        );
        reg.async_f_callback(process_message, process_message::process_message_api);
    }
}
