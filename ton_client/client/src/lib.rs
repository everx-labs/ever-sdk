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

extern crate api_info;
#[macro_use]
extern crate api_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

pub mod encoding;
pub mod error;
pub mod crypto;
pub mod tvm;
pub mod abi;
pub mod client;
pub mod boc;
pub mod processing;
pub mod utils;
mod api;
#[cfg(feature = "node_interaction")]
pub mod net;



#[cfg(test)]
mod tests;

pub use self::api::interop::*;
