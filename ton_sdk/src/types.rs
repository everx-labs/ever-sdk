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

use num_traits::cast::ToPrimitive;
use std::fmt;
use ton_types::Result;

use crate::error::SdkError;

pub const MESSAGES_TABLE_NAME: &str = "messages";
pub const CONTRACTS_TABLE_NAME: &str = "accounts";
pub const BLOCKS_TABLE_NAME: &str = "blocks";
pub const TRANSACTIONS_TABLE_NAME: &str = "transactions";

pub const MASTERCHAIN_ID: i32 = -1;

fn default_network_retries_count() -> i8 {
    5
}

fn default_message_retries_count() -> i8 {
    5
}

fn default_message_processing_timeout() -> u32 {
    40000
}

fn default_wait_for_timeout() -> u32 {
    40000
}

fn default_out_of_sync_threshold() -> i64 {
    15000
}

fn default_workchain() -> i32 {
    0
}

fn default_message_expiration_timeout() -> u32 {
    40000
}

fn default_message_expiration_timeout_grow_factor() -> f32 {
    1.5
}

#[derive(Serialize, Deserialize, Debug, Clone, ApiType)]
pub struct NetworkConfig {
    #[serde(default)]
    pub server_address: String,
    #[serde(default = "default_network_retries_count")]
    pub network_retries_count: i8,
    #[serde(default = "default_message_retries_count")]
    pub message_retries_count: i8,
    #[serde(default = "default_message_processing_timeout")]
    pub message_processing_timeout: u32,
    #[serde(default = "default_wait_for_timeout")]
    pub wait_for_timeout: u32,
    #[serde(default = "default_out_of_sync_threshold")]
    pub out_of_sync_threshold: i64,
    pub access_key: Option<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            server_address: String::new(),
            network_retries_count: default_network_retries_count(),
            message_retries_count: default_message_retries_count(),
            message_processing_timeout: default_message_processing_timeout(),
            wait_for_timeout: default_wait_for_timeout(),
            out_of_sync_threshold: default_out_of_sync_threshold(),
            access_key: None,
        }
    }
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

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct StringId(String);

pub type BlockId = StringId;

impl From<String> for StringId {
    fn from(id: String) -> Self {
        StringId { 0: id }
    }
}

impl From<&str> for StringId {
    fn from(id: &str) -> Self {
        StringId { 0: id.to_owned() }
    }
}

impl From<Vec<u8>> for StringId {
    fn from(id: Vec<u8>) -> Self {
        StringId { 0: hex::encode(id) }
    }
}

impl From<&[u8]> for StringId {
    fn from(id: &[u8]) -> Self {
        StringId { 0: hex::encode(id) }
    }
}

impl fmt::Display for StringId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl StringId {
    pub fn to_base64(&self) -> Result<String> {
        let bytes = self.to_bytes()?;
        Ok(base64::encode(&bytes))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        hex::decode(&self.0).map_err(Into::into)
    }
}

pub fn grams_to_u64(grams: &ton_block::types::Grams) -> Result<u64> {
    grams.0.to_u64().ok_or(
        SdkError::InvalidData {
            msg: "Cannot convert grams value".to_owned(),
        }
        .into(),
    )
}
