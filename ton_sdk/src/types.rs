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

pub const MESSAGES_TABLE_NAME: &str = "messages";
pub const CONTRACTS_TABLE_NAME: &str = "accounts";
pub const BLOCKS_TABLE_NAME: &str = "blocks";
pub const TRANSACTIONS_TABLE_NAME: &str = "transactions";

pub const DEFAULT_RETRIES_COUNT: u8 = 5;
pub const DEFAULT_EXPIRATION_TIMEOUT: u32 = 10000;
pub const DEFAULT_PROCESSING_TIMEOUT: u32 = 40000;
pub const DEFAULT_TIMEOUT_GROW_FACTOR: f32 = 1.5;
pub const DEFAULT_WAIT_TIMEOUT: u32 = 40000;


#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct TimeoutsConfig {
    pub message_retries_count: u8,
    pub message_expiration_timeout: u32,
    pub message_expiration_timeout_grow_factor: f32,
    pub message_processing_timeout: u32,
    pub message_processing_timeout_grow_factor: f32,
    pub wait_for_timeout: u32,
}

impl Default for TimeoutsConfig {
    fn default() -> Self {
        Self {
            message_retries_count: DEFAULT_RETRIES_COUNT,
            message_expiration_timeout: DEFAULT_EXPIRATION_TIMEOUT,
            message_expiration_timeout_grow_factor: DEFAULT_TIMEOUT_GROW_FACTOR,
            message_processing_timeout: DEFAULT_PROCESSING_TIMEOUT,
            message_processing_timeout_grow_factor: DEFAULT_TIMEOUT_GROW_FACTOR,
            wait_for_timeout: DEFAULT_WAIT_TIMEOUT,
        }
    }
}

// Represents config to connect node
#[cfg(feature = "node_interaction")]
#[derive(Debug, Deserialize, Serialize)]
pub struct NodeClientConfig {
    pub base_url: Option<String>,
    pub timeouts: Option<TimeoutsConfig>,
    pub access_key: Option<String>,
}

#[cfg(not(feature = "node_interaction"))]
#[derive(Debug, Deserialize, Serialize)]
pub struct NodeClientConfig {
    pub timeouts: Option<TimeoutsConfig>,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct StringId (String);

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