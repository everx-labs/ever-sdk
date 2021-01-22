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
    /// DApp Server public address. 
    /// For instance, for `net.ton.dev/graphql` GraphQL endpoint the server address will be net.ton.dev 
    pub server_address: Option<String>,

    /// List of DApp Server addresses. Any correct URL format can be specified, including IP addresses
    /// This parameter is prevailing over `server_address`.
    pub endpoints: Option<Vec<String>>,

    /// The number of automatic network retries that SDK performs in case of connection problems
    /// The default value is 5.
    #[serde(default = "default_network_retries_count",
    deserialize_with = "deserialize_network_retries_count")]
    pub network_retries_count: i8,

    /// The number of automatic message processing retries that SDK performs
    /// in case of `Message Expired (507)` error - but only for those messages which 
    /// local emulation was successfull or failed with replay protection error.
    /// The default value is 5.
    #[serde(default = "default_message_retries_count",
    deserialize_with = "deserialize_message_retries_count")]
    pub message_retries_count: i8,

    /// Timeout that is used to process message delivery for the contracts
    /// which ABI does not include "expire" header.
    /// If the message is not delivered within the speficied timeout 
    /// the appropriate error occurs.
    #[serde(default = "default_message_processing_timeout",
    deserialize_with = "deserialize_message_processing_timeout")]
    pub message_processing_timeout: u32,

    /// Maximum timeout that is used for query response.
    /// The default value is 40 sec.
    #[serde(default = "default_wait_for_timeout",
    deserialize_with = "deserialize_wait_for_timeout")]
    pub wait_for_timeout: u32,

    /// Maximum time difference between server and client. If client's device time is out of sinc and difference is more than 
    /// the threshold then error will occur. Also an error will occur if the specified threshold is more than 
    /// `message_processing_timeout/2`. 
    /// The default value is 15 sec.
    #[serde(default = "default_out_of_sync_threshold",
    deserialize_with = "deserialize_out_of_sync_threshold")]
    pub out_of_sync_threshold: u32,

    /// Timeout between reconnect attempts
    #[serde(default = "default_reconnect_timeout",
    deserialize_with = "deserialize_reconnect_timeout")]
    pub reconnect_timeout: u32,

    /// Access key to GraphQL API. At the moment is not used in production
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
