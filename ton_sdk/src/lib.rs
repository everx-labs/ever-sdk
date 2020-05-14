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

#[macro_use]
extern crate ton_vm;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_json;

#[cfg(feature = "fee_calculation")]
extern crate ton_executor;

use ton_types::Result;

pub use ton_abi::json_abi;
pub use ton_abi::Contract as AbiContract;
pub use ton_abi::Function as AbiFunction;

mod error;
pub use error::SdkError;

mod contract;
pub use contract::{Contract, ContractImage, FunctionCallSet};

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
pub use node_client::OrderBy;
pub use node_client::NodeClient;

#[cfg(not(feature = "node_interaction"))]
pub mod node_client {
    use crate::{NodeClientConfig, TimeoutsConfig};
    use ton_types::Result;

    pub struct NodeClient {
        timeouts: TimeoutsConfig
    }

    impl NodeClient {
        // Globally initializes client with server address
        pub fn new(config: NodeClientConfig) -> Result<NodeClient> {
            Ok(NodeClient {
                timeouts: config.timeouts.unwrap_or_default()
            })
        }

        pub fn timeouts(&self) -> &TimeoutsConfig {
            &self.timeouts
        }
    }
}

pub mod json_helper;

/// Init SKD. Globally saves queries and requests server URLs
pub fn init(config: NodeClientConfig) -> Result<NodeClient> {
    NodeClient::new(config)
}

/// Init SKD. Globally saves queries and requests server URLs
pub fn init_json(config: &str) -> Result<NodeClient> {
    init(serde_json::from_str(config)
        .map_err(|err| SdkError::InvalidArg { msg: format!("{}", err) } )?)
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
