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

pub(crate) mod decode_message;
pub(crate) mod encode_account;
pub(crate) mod encode_message;
mod errors;
mod internal;
mod signing;
mod types;

pub use decode_message::{
    decode_message, decode_message_body, DecodedMessageBody, MessageBodyType,
    ParamsOfDecodeMessage, ParamsOfDecodeMessageBody,
};
pub use encode_account::{
    encode_account, ParamsOfEncodeAccount, ResultOfEncodeAccount, StateInitParams, StateInitSource,
};
pub use encode_message::{
    attach_signature, attach_signature_to_message_body, encode_message, encode_message_body,
    CallSet, DeploySet, ParamsOfAttachSignature, ParamsOfAttachSignatureToMessageBody,
    ParamsOfEncodeMessage, ParamsOfEncodeMessageBody, ResultOfAttachSignature,
    ResultOfAttachSignatureToMessageBody, ResultOfEncodeMessage, ResultOfEncodeMessageBody,
};
pub use errors::{Error, ErrorCode};
pub use signing::Signer;
pub use types::{Abi, AbiHandle, FunctionHeader, MessageSource};

pub const DEFAULT_WORKCHAIN: i32 = 0;
