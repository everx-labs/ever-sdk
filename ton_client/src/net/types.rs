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

use serde::{Deserialize, Deserializer};

pub const MESSAGES_TABLE_NAME: &str = "messages";
pub const CONTRACTS_TABLE_NAME: &str = "accounts";
pub const BLOCKS_TABLE_NAME: &str = "blocks";
pub const TRANSACTIONS_TABLE_NAME: &str = "transactions";

pub fn default_network_retries_count() -> i8 {
    5
}

pub fn default_message_retries_count() -> i8 {
    5
}

pub fn default_message_processing_timeout() -> u32 {
    40000
}

pub fn default_wait_for_timeout() -> u32 {
    40000
}

pub fn default_out_of_sync_threshold() -> u32 {
    15000
}

pub fn default_reconnect_timeout() -> u32 {
    1000
}

fn deserialize_network_retries_count<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<i8, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_network_retries_count()))
}

fn deserialize_message_retries_count<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<i8, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_message_retries_count()))
}

fn deserialize_message_processing_timeout<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_message_processing_timeout()))
}

fn deserialize_wait_for_timeout<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_wait_for_timeout()))
}

fn deserialize_out_of_sync_threshold<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_out_of_sync_threshold()))
}

fn deserialize_reconnect_timeout<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_reconnect_timeout()))
}

#[derive(Serialize, Deserialize, Debug, Clone, ApiType)]
pub struct NetworkConfig {
    pub server_address: Option<String>,
    pub endpoints: Option<Vec<String>>,
    #[serde(default = "default_network_retries_count",
    deserialize_with = "deserialize_network_retries_count")]
    pub network_retries_count: i8,
    #[serde(default = "default_message_retries_count",
    deserialize_with = "deserialize_message_retries_count")]
    pub message_retries_count: i8,
    #[serde(default = "default_message_processing_timeout",
    deserialize_with = "deserialize_message_processing_timeout")]
    pub message_processing_timeout: u32,
    #[serde(default = "default_wait_for_timeout",
    deserialize_with = "deserialize_wait_for_timeout")]
    pub wait_for_timeout: u32,
    #[serde(default = "default_out_of_sync_threshold",
    deserialize_with = "deserialize_out_of_sync_threshold")]
    pub out_of_sync_threshold: u32,
    #[serde(default = "default_reconnect_timeout",
    deserialize_with = "deserialize_reconnect_timeout")]
    pub reconnect_timeout: u32,
    pub access_key: Option<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            server_address: None,
            endpoints: None,
            network_retries_count: default_network_retries_count(),
            message_retries_count: default_message_retries_count(),
            message_processing_timeout: default_message_processing_timeout(),
            wait_for_timeout: default_wait_for_timeout(),
            out_of_sync_threshold: default_out_of_sync_threshold(),
            reconnect_timeout: default_reconnect_timeout(),
            access_key: None,
        }
    }
}
