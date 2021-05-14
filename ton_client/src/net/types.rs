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

pub fn default_sending_endpoint_count() -> u8 {
    2
}

pub fn default_max_reconnect_timeout() -> u32 {
    120000
}

pub fn default_reconnect_timeout() -> u32 {
    1000
}

pub fn default_latency_detection_frequency() -> u32 {
    60000
}

pub fn default_max_latency() -> u32 {
    60000
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

fn deserialize_sending_endpoint_count<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u8, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_sending_endpoint_count()))
}

fn deserialize_max_reconnect_timeout<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_max_reconnect_timeout()))
}

fn deserialize_reconnect_timeout<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_reconnect_timeout()))
}

fn deserialize_max_latency<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_max_latency()))
}

fn deserialize_latency_detection_frequency<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_latency_detection_frequency()))
}

#[derive(Serialize, Deserialize, Debug, Clone, ApiType)]
pub struct NetworkConfig {
    /// DApp Server public address.
    /// For instance, for `net.ton.dev/graphql` GraphQL endpoint the server address will be net.ton.dev
    pub server_address: Option<String>,

    /// List of DApp Server addresses. Any correct URL format can be specified, including IP addresses
    /// This parameter is prevailing over `server_address`.
    pub endpoints: Option<Vec<String>>,

    /// Deprecated. You must use `network.max_reconnect_timeout` that allows to specify maximum network resolving timeout.
    #[serde(
        default = "default_network_retries_count",
        deserialize_with = "deserialize_network_retries_count"
    )]
    pub network_retries_count: i8,

    /// Maximum time for sequential reconnections.
    ///
    /// Must be specified in milliseconds. Default is 120000 (2 min).
    #[serde(
        default = "default_max_reconnect_timeout",
        deserialize_with = "deserialize_max_reconnect_timeout"
    )]
    pub max_reconnect_timeout: u32,

    /// Deprecated
    #[serde(
        default = "default_reconnect_timeout",
        deserialize_with = "deserialize_reconnect_timeout"
    )]
    pub reconnect_timeout: u32,

    /// The number of automatic message processing retries that SDK performs
    /// in case of `Message Expired (507)` error - but only for those messages which
    /// local emulation was successful or failed with replay protection error.
    ///
    /// Default is 5.
    #[serde(
        default = "default_message_retries_count",
        deserialize_with = "deserialize_message_retries_count"
    )]
    pub message_retries_count: i8,

    /// Timeout that is used to process message delivery for the contracts
    /// which ABI does not include "expire" header.
    /// If the message is not delivered within the specified timeout
    /// the appropriate error occurs.
    ///
    /// Must be specified in milliseconds. Default is 40000 (40 sec).
    #[serde(
        default = "default_message_processing_timeout",
        deserialize_with = "deserialize_message_processing_timeout"
    )]
    pub message_processing_timeout: u32,

    /// Maximum timeout that is used for query response.
    ///
    /// Must be specified in milliseconds. Default is 40000 (40 sec).
    #[serde(
        default = "default_wait_for_timeout",
        deserialize_with = "deserialize_wait_for_timeout"
    )]
    pub wait_for_timeout: u32,

    /// Maximum time difference between server and client. If client's device time is out of sync and difference is more than
    /// the threshold then error will occur. Also an error will occur if the specified threshold is more than
    /// `message_processing_timeout/2`.
    ///
    /// Must be specified in milliseconds. Default is 15000 (15 sec).
    #[serde(
        default = "default_out_of_sync_threshold",
        deserialize_with = "deserialize_out_of_sync_threshold"
    )]
    pub out_of_sync_threshold: u32,

    /// Maximum number of randomly chosen endpoints the library uses to send message.
    ///
    /// Default is 2.
    #[serde(
        default = "default_sending_endpoint_count",
        deserialize_with = "deserialize_sending_endpoint_count"
    )]
    pub sending_endpoint_count: u8,

    /// Frequency of sync latency detection. Library periodically performs
    /// checking for the server sync latency on current endpoint.
    /// If the latency is less then the maximum allowed then library
    /// selects new current endpoint.
    ///
    /// Must be specified in milliseconds. Default is 60000 (1 min).
    #[serde(
        default = "default_latency_detection_frequency",
        deserialize_with = "deserialize_latency_detection_frequency"
    )]
    pub latency_detection_interval: u32,

    /// Maximum value for the server sync latency. Library periodically performs
    /// checking for the server sync latency on current endpoint.
    /// If the latency is less then the maximum allowed then library
    /// selects new current endpoint.
    ///
    /// Must be specified in milliseconds. Default is 60000 (1 min).
    #[serde(
        default = "default_max_latency",
        deserialize_with = "deserialize_max_latency"
    )]
    pub max_latency: u32,

    /// Access key to GraphQL API. At the moment is not used in production.
    pub access_key: Option<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            server_address: None,
            endpoints: None,
            network_retries_count: default_network_retries_count(),
            max_reconnect_timeout: default_max_reconnect_timeout(),
            reconnect_timeout: default_reconnect_timeout(),
            message_retries_count: default_message_retries_count(),
            message_processing_timeout: default_message_processing_timeout(),
            wait_for_timeout: default_wait_for_timeout(),
            out_of_sync_threshold: default_out_of_sync_threshold(),
            sending_endpoint_count: default_sending_endpoint_count(),
            latency_detection_interval: default_latency_detection_frequency(),
            max_latency: default_max_latency(),
            access_key: None,
        }
    }
}
