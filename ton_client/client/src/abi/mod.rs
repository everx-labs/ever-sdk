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
use ton_sdk::SdkAbiConfig;

fn default_workchain() -> i32 {
    0
}

fn default_message_expiration_timeout() -> u32 {
    40000
}

fn default_message_expiration_timeout_grow_factor() -> f32 {
    1.5
}

#[derive(Deserialize, Debug, Clone, ApiType)]
pub struct AbiConfig {
    #[serde(default = "default_workchain")]
    pub workchain: i32,
    #[serde(default = "default_message_expiration_timeout")]
    pub message_expiration_timeout: u32,
    #[serde(default = "default_message_expiration_timeout_grow_factor")]
    pub message_expiration_timeout_grow_factor: f32,
}

impl AbiConfig {
    pub fn to_sdk(&self) -> SdkAbiConfig {
        SdkAbiConfig {
            workchain: self.workchain,
            message_expiration_timeout_grow_factor: self.message_expiration_timeout_grow_factor,
            message_expiration_timeout: self.message_expiration_timeout,
        }
    }
}

impl Default for AbiConfig {
    fn default() -> Self {
        Self {
            workchain: default_workchain(),
            message_expiration_timeout: default_message_expiration_timeout(),
            message_expiration_timeout_grow_factor: default_message_expiration_timeout_grow_factor(
            ),
        }
    }
}
