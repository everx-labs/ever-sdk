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

use std::fmt;
use ton_types::Result;
use num_traits::cast::ToPrimitive;

use crate::error::SdkError;

pub const MESSAGES_TABLE_NAME: &str = "messages";
pub const CONTRACTS_TABLE_NAME: &str = "accounts";
pub const BLOCKS_TABLE_NAME: &str = "blocks";
pub const TRANSACTIONS_TABLE_NAME: &str = "transactions";

pub const DEFAULT_RETRIES_COUNT: u8 = 5;
pub const DEFAULT_EXPIRATION_TIMEOUT: u32 = 40000;
pub const DEFAULT_PROCESSING_TIMEOUT: u32 = 40000;
pub const DEFAULT_TIMEOUT_GROW_FACTOR: f32 = 1.5;
pub const DEFAULT_WAIT_TIMEOUT: u32 = 40000;
pub const DEFAULT_OUT_OF_SYNC_THRESHOLD: i64 = 15000;

pub const MASTERCHAIN_ID: i32 = -1;


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NetworkConfig {
    pub server_address: String,
    pub message_retries_count: Option<u8>,
    pub message_processing_timeout: Option<u32>,
    pub wait_for_timeout: Option<u32>,
    pub out_of_sync_threshold: Option<i64>,
    pub access_key: Option<String>,
}

impl NetworkConfig {
    pub fn server_address(&self) -> &str {
        &self.server_address
    }

    pub fn message_retries_count(&self) -> u8 {
        self.message_retries_count.unwrap_or(DEFAULT_RETRIES_COUNT)
    }

    pub fn message_processing_timeout(&self) -> u32 {
        self.message_processing_timeout.unwrap_or(DEFAULT_PROCESSING_TIMEOUT)
    }

    pub fn wait_for_timeout(&self) -> u32 {
        self.wait_for_timeout.unwrap_or(DEFAULT_WAIT_TIMEOUT)
    }

    pub fn out_of_sync_threshold(&self) -> i64 {
        self.out_of_sync_threshold.unwrap_or(DEFAULT_OUT_OF_SYNC_THRESHOLD)
    }

    pub fn access_key(&self) -> Option<&str> {
        self.access_key.as_ref().map(|string| string.as_str())
    }
}

#[derive(Deserialize, Debug, Default, Clone, ApiType)]
pub struct AbiConfig {
    message_expiration_timeout: Option<u32>,
    message_expiration_timeout_grow_factor: Option<f32>,
}

impl AbiConfig {
    pub fn message_expiration_timeout(&self) -> u32 {
        self.message_expiration_timeout.unwrap_or(DEFAULT_EXPIRATION_TIMEOUT)
    }

    pub fn message_expiration_timeout_grow_factor(&self) -> f32 {
        self.message_expiration_timeout_grow_factor.unwrap_or(DEFAULT_TIMEOUT_GROW_FACTOR)
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct StringId (String);

pub type BlockId = StringId;

impl From<String> for StringId {
    fn from(id: String) -> Self {
        StringId{0: id}
    }
}

impl From<&str> for StringId {
    fn from(id: &str) -> Self {
        StringId{0: id.to_owned()}
    }
}

impl From<Vec<u8>> for StringId {
    fn from(id: Vec<u8>) -> Self {
        StringId{0: hex::encode(id)}
    }
}

impl From<&[u8]> for StringId {
    fn from(id: &[u8]) -> Self {
        StringId{0: hex::encode(id)}
    }
}

impl fmt::Display for StringId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}", self.0)
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
    grams.0.to_u64()
        .ok_or(SdkError::InvalidData { msg: "Cannot convert grams value".to_owned() }.into())
}
