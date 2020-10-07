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

#[cfg(test)]
mod tests;

mod abi;
pub(crate) mod decode;
pub(crate) mod encode;
mod errors;
mod internal;
mod signing;

pub use abi::{Abi, AbiHandle, FunctionHeader};
pub use decode::{decode_message, DecodedMessageBody, DecodedMessageType, ParamsOfDecodeMessage};
pub use encode::{
    attach_signature, encode_message, CallSet, DeploySet, ParamsOfAttachSignature,
    ParamsOfEncodeMessage, ResultOfAttachSignature, ResultOfEncodeMessage,
};
pub use errors::{Error, ErrorCode};
pub use signing::Signer;

pub const DEFAULT_WORKCHAIN: i32 = 0;
