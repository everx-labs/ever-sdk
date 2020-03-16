/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

#![recursion_limit="128"] // needed for error_chain

extern crate ton_block;
extern crate ton_types;
#[macro_use]
extern crate ton_vm;
extern crate ton_abi;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate hex;
extern crate ed25519_dalek;
extern crate sha2;
extern crate base64;
extern crate chrono;
#[macro_use]
extern crate failure;
extern crate crc_any;
extern crate num_traits;

#[cfg(feature = "node_interaction")]
#[macro_use]
extern crate lazy_static;
#[cfg(feature = "node_interaction")]
#[macro_use]
extern crate serde_json;
#[cfg(feature = "node_interaction")]
extern crate futures;
#[cfg(feature = "node_interaction")]
extern crate graphite;
#[cfg(feature = "node_interaction")]
extern crate reqwest;

#[cfg(feature = "fee_calculation")]
extern crate ton_executor;

pub use ton_abi::json_abi;
pub use ton_abi::Contract as AbiContract;
pub use ton_abi::Function as AbiFunction;

#[allow(deprecated)]
#[macro_use]
mod error;
pub use error::*;

mod contract;
pub use contract::*;

mod message;
pub use message::*;

mod local_tvm;
#[cfg(feature = "fee_calculation")]
pub use local_tvm::executor::TransactionFees;

#[cfg(feature = "node_interaction")]
mod transaction;
#[cfg(feature = "node_interaction")]
pub use transaction::*;
/*
#[cfg(feature = "node_interaction")]
mod block;
#[cfg(feature = "node_interaction")]
pub use block::*;
*/
mod types;
pub use types::*;

#[cfg(feature = "node_interaction")]
pub mod queries_helper;

pub mod json_helper;


/// Init SKD. Globally saves queries and requests server URLs
#[cfg(feature = "node_interaction")]
pub fn init(config: NodeClientConfig) -> SdkResult<()> { 
    queries_helper::init(config)
}

/// Init SKD. Globally saves queries and requests server URLs
#[cfg(feature = "node_interaction")]
pub fn init_json(config: &str) -> SdkResult<()> {
    init(serde_json::from_str(config)
        .map_err(|err| SdkErrorKind::InvalidArg { msg: format!("{}", err) } )?)
}

/// Uninit SKD. Should be called before process
#[cfg(feature = "node_interaction")]
pub fn uninit() {
    queries_helper::uninit();
}

#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate dirs;

#[cfg(test)]
#[path = "tests/test_lib.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/test_piggy_bank.rs"]
mod test_piggy_bank;

#[cfg(test)]
#[path = "tests/tests_common.rs"]
mod tests_common;
