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

#[macro_use]
extern crate ton_vm;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;

#[cfg(feature = "node_interaction")]
#[macro_use]
extern crate serde_json;

#[cfg(feature = "fee_calculation")]
extern crate ton_executor;

pub use ton_abi::json_abi;
pub use ton_abi::Contract as AbiContract;
pub use ton_abi::Function as AbiFunction;

mod error;
pub use error::{SdkError, SdkErrorKind, SdkResult};

mod contract;
pub use contract::{Contract, ContractImage};

mod message;
pub use message::{Message, MessageId, MessageType};

mod local_tvm;
#[cfg(feature = "fee_calculation")]
pub use local_tvm::executor::TransactionFees;

#[cfg(feature = "node_interaction")]
mod transaction;
#[cfg(feature = "node_interaction")]
pub use transaction::{Transaction, TransactionId};

pub mod types;
pub use types::{NodeClientConfig, TimeoutsConfig};

#[cfg(feature = "node_interaction")]
pub mod node_client;
#[cfg(feature = "node_interaction")]
pub use node_client::{NodeClient, OrderBy};

pub mod json_helper;


/// Init SKD. Globally saves queries and requests server URLs
#[cfg(feature = "node_interaction")]
pub fn init(config: NodeClientConfig) -> SdkResult<NodeClient> { 
    NodeClient::new(config)
}

/// Init SKD. Globally saves queries and requests server URLs
#[cfg(feature = "node_interaction")]
pub fn init_json(config: &str) -> SdkResult<NodeClient> {
    init(serde_json::from_str(config)
        .map_err(|err| SdkErrorKind::InvalidArg { msg: format!("{}", err) } )?)
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
