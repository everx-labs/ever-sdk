/*
* Copyright 2018-2021 EverX Labs Ltd.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific EVERX DEV software governing permissions and
* limitations under the License.
*/

use serde::{Deserialize, Deserializer};

pub const MESSAGES_COLLECTION: &str = "messages";
pub const ACCOUNTS_COLLECTION: &str = "accounts";
pub const BLOCKS_COLLECTION: &str = "blocks";
pub const TRANSACTIONS_COLLECTION: &str = "transactions";

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

pub fn default_sending_endpoint_count() -> u8 {
    1
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

pub fn default_query_timeout() -> u32 {
    60000
}

pub fn default_queries_protocol() -> NetworkQueriesProtocol {
    NetworkQueriesProtocol::HTTP
}

pub fn default_first_remp_status_timeout() -> u32 {
    1
}

pub fn default_next_remp_status_timeout() -> u32 {
    5000
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

fn deserialize_max_latency<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_max_latency()))
}

fn deserialize_latency_detection_frequency<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_latency_detection_frequency()))
}

fn deserialize_query_timeout<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_query_timeout()))
}

fn deserialize_queries_protocol<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<NetworkQueriesProtocol, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_queries_protocol()))
}

fn deserialize_first_remp_status_timeout<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_first_remp_status_timeout()))
}

fn deserialize_next_remp_status_timeout<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_next_remp_status_timeout()))
}

#[derive(Debug, Clone, PartialEq, ApiType)]
pub struct TrustedMcBlockId {
    /// Trusted key-block sequence number
    pub seq_no: u32,

    /// Trusted key-block root hash, encoded as HEX
    pub root_hash: String,
}

/// Network protocol used to perform GraphQL queries.
#[derive(Serialize, Deserialize, Debug, Clone, ApiType)]
pub enum NetworkQueriesProtocol {
    /// Each GraphQL query uses separate HTTP request.
    HTTP,

    /// All GraphQL queries will be served using single web socket connection.
    /// SDK is tested to reliably handle 5000 parallel network requests (sending and processing
    /// messages, quering and awaiting blockchain data)
    WS,
}

#[derive(Serialize, Deserialize, Debug, Clone, ApiType)]
pub struct NetworkConfig {
    /// **This field is deprecated, but left for backward-compatibility.** Evernode endpoint.
    pub server_address: Option<String>,

    /// List of Evernode endpoints. Any correct URL format can be specified, including IP addresses.
    /// This parameter is prevailing over `server_address`.
    /// Check the full list of [supported network endpoints](https://docs.evercloud.dev/products/evercloud/networks-endpoints).
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

    /// **DEPRECATED**: This parameter was deprecated.
    pub out_of_sync_threshold: Option<u32>,

    /// Maximum number of randomly chosen endpoints the library uses to broadcast a message.
    ///
    /// Default is 1.
    #[serde(
        default = "default_sending_endpoint_count",
        deserialize_with = "deserialize_sending_endpoint_count"
    )]
    pub sending_endpoint_count: u8,

    /// Frequency of sync latency detection. Library periodically
    /// checks the current endpoint for blockchain data synchronization latency.
    /// If the latency (time-lag) is less then `NetworkConfig.max_latency`
    /// then library selects another endpoint.
    ///
    /// Must be specified in milliseconds. Default is 60000 (1 min).
    #[serde(
        default = "default_latency_detection_frequency",
        deserialize_with = "deserialize_latency_detection_frequency"
    )]
    pub latency_detection_interval: u32,

    /// Maximum value for the endpoint's blockchain data synchronization latency (time-lag).
    /// Library periodically checks the current endpoint for blockchain
    /// data synchronization latency.
    /// If the latency (time-lag) is less then `NetworkConfig.max_latency`
    /// then library selects another endpoint.
    ///
    /// Must be specified in milliseconds. Default is 60000 (1 min).
    #[serde(
        default = "default_max_latency",
        deserialize_with = "deserialize_max_latency"
    )]
    pub max_latency: u32,

    /// Default timeout for http requests. Is is used when no timeout specified for the request to
    /// limit the answer waiting time. If no answer received during the timeout requests ends with
    /// error.
    ///
    /// Must be specified in milliseconds. Default is 60000 (1 min).
    #[serde(
        default = "default_query_timeout",
        deserialize_with = "deserialize_query_timeout"
    )]
    pub query_timeout: u32,

    /// Queries protocol. `HTTP` or `WS`.
    ///
    /// Default is `HTTP`.
    #[serde(
        default = "default_queries_protocol",
        deserialize_with = "deserialize_queries_protocol"
    )]
    pub queries_protocol: NetworkQueriesProtocol,

    /// UNSTABLE. First REMP status awaiting timeout. If no status received during the timeout than fallback
    /// transaction scenario is activated.
    ///
    /// Must be specified in milliseconds. Default is 1 (1 ms) in order to start fallback scenario
    /// together with REMP statuses processing while REMP is not properly tuned yet.
    #[serde(
        default = "default_first_remp_status_timeout",
        deserialize_with = "deserialize_first_remp_status_timeout"
    )]
    pub first_remp_status_timeout: u32,

    /// UNSTABLE. Subsequent REMP status awaiting timeout. If no status received during the timeout than fallback
    /// transaction scenario is activated.
    ///
    /// Must be specified in milliseconds. Default is 5000 (5 sec).
    #[serde(
        default = "default_next_remp_status_timeout",
        deserialize_with = "deserialize_next_remp_status_timeout"
    )]
    pub next_remp_status_timeout: u32,

    /// Network signature ID which is used by VM in signature verifying instructions if capability
    /// `CapSignatureWithId` is enabled in blockchain configuration parameters.
    ///
    /// This parameter should be set to `global_id` field from any blockchain block if network can
    /// not be reachable at the moment of message encoding and the message is aimed to be sent into
    /// network with `CapSignatureWithId` enabled. Otherwise signature ID is detected automatically
    /// inside message encoding functions
    pub signature_id: Option<i32>,

    /// Access key to GraphQL API (Project secret)
    pub access_key: Option<String>,
}

impl NetworkConfig {
    pub fn get_auth_header(&self) -> Option<(String, String)> {
        if let Some(key) = &self.access_key {
            let is_jwt = key.contains('.');
            let auth = if is_jwt {
                format!("Bearer {}", key)
            } else {
                format!("Basic {}", base64::encode(format!(":{}", key).as_bytes()))
            };
            Some(("Authorization".into(), auth))
        } else {
            None
        }
    }
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
            out_of_sync_threshold: None,
            sending_endpoint_count: default_sending_endpoint_count(),
            latency_detection_interval: default_latency_detection_frequency(),
            max_latency: default_max_latency(),
            query_timeout: default_query_timeout(),
            queries_protocol: default_queries_protocol(),
            first_remp_status_timeout: default_first_remp_status_timeout(),
            next_remp_status_timeout: default_next_remp_status_timeout(),
            signature_id: None,
            access_key: None,
        }
    }
}
