/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
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

#[cfg(test)]
#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

extern crate serde;
extern crate rand;
extern crate futures;
extern crate ed25519_dalek;
extern crate num_bigint;
extern crate sha2;
extern crate bip39;
extern crate hmac;
extern crate pbkdf2;
extern crate base58;
extern crate byteorder;
extern crate secp256k1;
extern crate ton_sdk;
extern crate tvm;
extern crate base64;

mod types;
mod dispatch;
mod client;
mod setup;
mod contracts;
mod crypto;

#[cfg(feature = "node_interaction")]
mod queries;

mod interop;

#[cfg(test)]
mod tests;

pub use self::interop::*;

