/*
* Copyright 2018-2021 TON Labs LTD.
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

pub mod abi;
pub mod boc;
pub mod client;
pub mod crypto;
pub mod debot;
pub mod encoding;
pub mod error;
pub mod json_interface;
pub mod net;
pub mod processing;
pub mod proofs;
pub mod tvm;
pub mod utils;

#[cfg(test)]
mod tests;
pub mod native;

pub use self::json_interface::interop::*;
pub use client::{ClientConfig, ClientContext};
