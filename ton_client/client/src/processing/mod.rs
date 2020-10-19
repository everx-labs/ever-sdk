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

#[cfg(test)]
mod tests;

mod blocks_walking;
mod errors;
mod fetching;
mod internal;
pub(crate) mod parsing;
pub(crate) mod process_message;
pub(crate) mod send_message;
mod types;
pub(crate) mod wait_for_transaction;

pub use errors::{Error, ErrorCode};
pub use process_message::{process_message, ParamsOfProcessMessage};
pub use send_message::{send_message, ParamsOfSendMessage, ResultOfSendMessage};
pub use types::{DecodedOutput, ProcessingEvent, ProcessingResponseType, ResultOfProcessMessage};
pub use wait_for_transaction::{wait_for_transaction, ParamsOfWaitForTransaction};

pub const DEFAULT_NETWORK_RETRIES_LIMIT: i8 = -1;
pub const DEFAULT_NETWORK_RETRIES_TIMEOUT: u32 = 1000;
pub const DEFAULT_EXPIRATION_RETRIES_LIMIT: i8 = 20;
pub const DEFAULT_EXPIRATION_RETRIES_TIMEOUT: u32 = 1000;
