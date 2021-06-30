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

pub(crate) mod decode_data;
pub(crate) mod decode_message;
pub(crate) mod encode_account;
pub(crate) mod encode_message;

mod errors;
mod internal;
mod signing;
mod types;

use serde::{Deserialize, Deserializer};

pub use decode_data::{decode_account_data, ParamsOfDecodeAccountData, ResultOfDecodeData};
pub use decode_message::{
    decode_message, decode_message_body, DecodedMessageBody, MessageBodyType,
    ParamsOfDecodeMessage, ParamsOfDecodeMessageBody,
};
pub use encode_account::{
    encode_account, ParamsOfEncodeAccount, ResultOfEncodeAccount, StateInitParams, StateInitSource,
};
pub use encode_message::{
    attach_signature, attach_signature_to_message_body, encode_internal_message, encode_message,
    encode_message_body, CallSet, DeploySet, ParamsOfAttachSignature,
    ParamsOfAttachSignatureToMessageBody, ParamsOfEncodeInternalMessage, ParamsOfEncodeMessage,
    ParamsOfEncodeMessageBody, ResultOfAttachSignature, ResultOfAttachSignatureToMessageBody,
    ResultOfEncodeInternalMessage, ResultOfEncodeMessage, ResultOfEncodeMessageBody,
};
pub use errors::{Error, ErrorCode};
pub use signing::Signer;
pub use types::{
    Abi, AbiContract, AbiData, AbiEvent, AbiFunction, AbiHandle, AbiParam, FunctionHeader,
    MessageSource,
};

pub fn default_workchain() -> i32 {
    0
}

pub fn default_message_expiration_timeout() -> u32 {
    40000
}

pub fn default_message_expiration_timeout_grow_factor() -> f32 {
    1.5
}

fn deserialize_workchain<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_workchain()))
}

fn deserialize_message_expiration_timeout<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_message_expiration_timeout()))
}

fn deserialize_message_expiration_timeout_grow_factor<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<f32, D::Error> {
    Ok(Option::deserialize(deserializer)?
        .unwrap_or(default_message_expiration_timeout_grow_factor()))
}

#[derive(Deserialize, Debug, Clone, ApiType)]
pub struct AbiConfig {
    /// Workchain id that is used by default in DeploySet
    #[serde(
        default = "default_workchain",
        deserialize_with = "deserialize_workchain"
    )]
    pub workchain: i32,

    /// Message lifetime for contracts which ABI includes "expire" header.
    /// The default value is 40 sec.
    #[serde(
        default = "default_message_expiration_timeout",
        deserialize_with = "deserialize_message_expiration_timeout"
    )]
    pub message_expiration_timeout: u32,

    /// Factor that increases the expiration timeout for each retry
    /// The default value is 1.5
    #[serde(
        default = "default_message_expiration_timeout_grow_factor",
        deserialize_with = "deserialize_message_expiration_timeout_grow_factor"
    )]
    pub message_expiration_timeout_grow_factor: f32,
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
