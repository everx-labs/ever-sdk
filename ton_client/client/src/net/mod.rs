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

use crate::client::ClientContext;
use crate::error::ClientResult;
use futures::{Future, FutureExt, StreamExt};
use rand::RngCore;
use std::collections::HashMap;
use tokio::sync::{
    mpsc::{channel, Sender},
    Mutex,
};

mod errors;
pub use errors::{Error, ErrorCode};

mod node_client;
pub(crate) use node_client::{NodeClient, MAX_TIMEOUT};
pub use node_client::{OrderBy, SortDirection};
use serde::{Deserialize, Deserializer};

#[cfg(test)]
mod tests;

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

pub fn default_out_of_sync_threshold() -> i64 {
    15000
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
) -> Result<i64, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_out_of_sync_threshold()))
}

#[derive(Serialize, Deserialize, Debug, Clone, ApiType)]
pub struct NetworkConfig {
    #[serde(default)]
    pub server_address: String,
    #[serde(
        default = "default_network_retries_count",
        deserialize_with = "deserialize_network_retries_count"
    )]
    pub network_retries_count: i8,
    #[serde(
        default = "default_message_retries_count",
        deserialize_with = "deserialize_message_retries_count"
    )]
    pub message_retries_count: i8,
    #[serde(
        default = "default_message_processing_timeout",
        deserialize_with = "deserialize_message_processing_timeout"
    )]
    pub message_processing_timeout: u32,
    #[serde(
        default = "default_wait_for_timeout",
        deserialize_with = "deserialize_wait_for_timeout"
    )]
    pub wait_for_timeout: u32,
    #[serde(
        default = "default_out_of_sync_threshold",
        deserialize_with = "deserialize_out_of_sync_threshold"
    )]
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

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfQueryCollection {
    /// Collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// Collection filter
    pub filter: Option<serde_json::Value>,
    /// Projection (result) string
    pub result: String,
    /// Sorting order
    pub order: Option<Vec<OrderBy>>,
    /// Number of documents to return
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ResultOfQueryCollection {
    /// Objects that match the provided criteria
    pub result: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, ApiType, Clone, Default)]
pub struct ParamsOfWaitForCollection {
    /// Collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// Collection filter
    pub filter: Option<serde_json::Value>,
    /// Projection (result) string
    pub result: String,
    /// Query timeout
    pub timeout: Option<u32>,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ResultOfWaitForCollection {
    /// First found object that matches the provided criteria
    pub result: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, num_derive::FromPrimitive)]
pub enum SubscriptionResponseType {
    Ok = 100,
    Error = 101,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfSubscribeCollection {
    /// Collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// Collection filter
    pub filter: Option<serde_json::Value>,
    /// Projection (result) string
    pub result: String,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ResultOfSubscribeCollection {
    /// Subscription handle. Must be closed with `unsubscribe`
    pub handle: u32,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ResultOfSubscription {
    /// First appeared object that matches the provided criteria
    pub result: serde_json::Value,
}

lazy_static! {
    static ref SUBSCRIPTIONS: Mutex<HashMap<u32, Sender<bool>>> = Mutex::new(HashMap::new());
}

async fn add_subscription_handle(handle: u32, aborter: Sender<bool>) {
    SUBSCRIPTIONS.lock().await.insert(handle, aborter);
}

async fn extract_subscription_handle(handle: &u32) -> Option<Sender<bool>> {
    SUBSCRIPTIONS.lock().await.remove(handle)
}

/// Queries collection data
///
/// Queries data that satisfies the `filter` conditions,
/// limits the number of returned records and orders them.
/// The projection fields are limited to  `result` fields
#[api_function]
pub async fn query_collection(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfQueryCollection,
) -> ClientResult<ResultOfQueryCollection> {
    let client = context.get_client()?;
    let result = client
        .query(
            &params.collection,
            &params.filter.unwrap_or(json!({})),
            &params.result,
            params.order,
            params.limit,
            None,
        )
        .await
        .map_err(|err| Error::queries_query_failed(err).add_network_url(client))?;

    let result = serde_json::from_value(result).map_err(|err| {
        Error::queries_query_failed(format!("Can not parse result: {}", err))
            .add_network_url(client)
    })?;

    Ok(ResultOfQueryCollection { result })
}

/// Returns an object that fulfills the conditions or waits for its appearance
///
/// Triggers only once.
/// If object that satisfies the `filter` conditions
/// already exists - returns it immediately.
/// If not - waits for insert/update of data withing the specified `timeout`,
/// and returns it.
/// The projection fields are limited to  `result` fields
#[api_function]
pub async fn wait_for_collection(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfWaitForCollection,
) -> ClientResult<ResultOfWaitForCollection> {
    let client = context.get_client()?;
    let result = client
        .wait_for(
            &params.collection,
            &params.filter.unwrap_or(json!({})),
            &params.result,
            params.timeout,
        )
        .await
        .map_err(|err| Error::queries_wait_for_failed(err).add_network_url(client))?;

    Ok(ResultOfWaitForCollection { result })
}

pub async fn subscribe_collection<F: Future<Output = ()> + Send + Sync>(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfSubscribeCollection,
    callback: impl Fn(ClientResult<ResultOfSubscription>) -> F + Send + Sync + 'static,
) -> ClientResult<ResultOfSubscribeCollection> {
    let handle = rand::thread_rng().next_u32();

    let client = context.get_client()?;
    let subscription = client
        .subscribe(
            &params.collection,
            &params.filter.unwrap_or(json!({})),
            &params.result,
        )
        .await
        .map_err(|err| Error::queries_subscribe_failed(err).add_network_url(client))?;

    let (sender, mut receiver) = channel(1);

    add_subscription_handle(handle, sender).await;

    // spawn thread which reads subscription stream and calls callback with data
    context.env.spawn(Box::pin(async move {
        let wait_abortion = receiver.recv().fuse();
        futures::pin_mut!(wait_abortion);
        let mut data_stream = subscription.data_stream.fuse();
        loop {
            futures::select!(
                // waiting next subscription data
                data = data_stream.select_next_some() => {
                    callback(data.map(|data| ResultOfSubscription { result: data })).await
                },
                // waiting for unsubscribe
                _ = wait_abortion => break
            );
        }
        subscription.unsubscribe.await;
    }));

    Ok(ResultOfSubscribeCollection { handle })
}

/// Cancels a subscription
///
/// Cancels a subscription specified by its handle.
#[api_function]
pub async fn unsubscribe(
    _context: std::sync::Arc<ClientContext>,
    params: ResultOfSubscribeCollection,
) -> ClientResult<()> {
    if let Some(mut sender) = extract_subscription_handle(&params.handle).await {
        let _ = sender.send(true);
    }

    Ok(())
}
