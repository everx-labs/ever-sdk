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
*/

use crate::dispatch::DispatchTable;

#[cfg(test)]
mod tests;

mod internal;

pub mod encode;
pub mod abi;
pub mod decode;
pub mod signing;
pub mod defaults;

pub use abi::{Abi, AbiHandle};
pub use signing::{MessageSigning};

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.call("abi.encode_message", encode::encode_message);
    handlers.call("abi.attach_signature", encode::attach_signature);
}
