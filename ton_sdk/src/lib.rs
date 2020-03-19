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

#[macro_use]
extern crate ton_vm;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;

#[cfg(feature = "node_interaction")]
#[macro_use]
extern crate lazy_static;
#[cfg(feature = "node_interaction")]
#[macro_use]
extern crate serde_json;

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
pub use local_tvm::*;

#[cfg(feature = "node_interaction")]
mod transaction;
#[cfg(feature = "node_interaction")]
pub use transaction::*;

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
#[path = "tests/test_lib.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/test_piggy_bank.rs"]
mod test_piggy_bank;

#[cfg(test)]
#[path = "tests/tests_common.rs"]
mod tests_common;
