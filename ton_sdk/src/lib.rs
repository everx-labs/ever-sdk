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

extern crate api_info;
#[macro_use]
extern crate api_derive;

pub use ton_abi::json_abi;
pub use ton_abi::Contract as AbiContract;
pub use ton_abi::Function as AbiFunction;

mod error;
pub use error::SdkError;

mod contract;
pub use contract::{
    Contract, ContractImage, FunctionCallSet, LocalRunContext, MessageProcessingState,
    ReceivedTransaction, SdkMessage,
};

mod message;
pub use message::{Message, MessageId, MessageType};

mod local_tvm;
pub use local_tvm::{call_tvm, call_tvm_stack};

mod transaction;
pub use transaction::{Transaction, TransactionFees, TransactionId};

mod block;
pub use block::{Block, MsgDescr};

pub mod types;
pub use types::{SdkAbiConfig, BlockId};

pub mod json_helper;

#[cfg(test)]
#[path = "tests/test_lib.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/test_piggy_bank.rs"]
mod test_piggy_bank;

#[cfg(test)]
#[path = "tests/tests_common.rs"]
mod tests_common;
