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
use crate::dispatch::{Callback, ModuleReg, Registrar};
use crate::error::ApiResult;
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
pub use node_client::{NetworkConfig, OrderBy, SortDirection};
pub(crate) use node_client::{NodeClient, MAX_TIMEOUT};

#[cfg(test)]
mod tests;

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfQueryCollection {
    /// collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// collection filter
    pub filter: Option<serde_json::Value>,
    /// projection (result) string
    pub result: String,
    /// sorting order
    pub order: Option<Vec<OrderBy>>,
    /// number of documents to return
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ResultOfQueryCollection {
    /// objects that match provided criteria
    pub result: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, ApiType, Clone, Default)]
pub struct ParamsOfWaitForCollection {
    /// collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// collection filter
    pub filter: Option<serde_json::Value>,
    /// projection (result) string
    pub result: String,
    /// query timeout
    pub timeout: Option<u32>,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ResultOfWaitForCollection {
    /// first found object that match provided criteria
    pub result: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, num_derive::FromPrimitive)]
pub enum SubscriptionResponseType {
    Ok = 100,
    Error = 101
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfSubscribeCollection {
    /// collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// collection filter
    pub filter: Option<serde_json::Value>,
    /// projection (result) string
    pub result: String,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ResultOfSubscribeCollection {
    /// handle to subscription. It then can be used in `get_next_subscription_data` function
    /// and must be closed with `unsubscribe`
    pub handle: u32,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ResultOfSubscription {
    /// first appeared object that match provided criteria
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

#[api_function]
pub async fn query_collection(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfQueryCollection,
) -> ApiResult<ResultOfQueryCollection> {
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

#[api_function]
pub async fn wait_for_collection(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfWaitForCollection,
) -> ApiResult<ResultOfWaitForCollection> {
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

#[api_function]
pub(crate) async fn subscribe_collection(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfSubscribeCollection,
    callback: std::sync::Arc<Callback>,
) -> ApiResult<ResultOfSubscribeCollection> {
    let callback = move |result: ApiResult<ResultOfSubscription>| {
        match result {
            Ok(result) => callback.call(result, SubscriptionResponseType::Ok as u32),
            Err(err) => callback.call(err, SubscriptionResponseType::Error as u32)
        }
        futures::future::ready(())
    };

    subscribe_collection_rust(context, params, callback).await
}

pub async fn subscribe_collection_rust<F: Future<Output = ()> + Send + Sync>(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfSubscribeCollection,
    callback: impl Fn(ApiResult<ResultOfSubscription>) -> F + Send + Sync + 'static
) -> ApiResult<ResultOfSubscribeCollection> {
    let handle = rand::thread_rng().next_u32();

    let client = context.get_client()?;
    let subscription = client
        .subscribe(
            &params.collection,
            &params.filter.unwrap_or(json!({})),
            &params.result,
        )
        .await
        .map_err(|err| Error::queries_wait_for_failed(err).add_network_url(client))?;

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

#[api_function]
pub async fn unsubscribe(
    _context: std::sync::Arc<ClientContext>,
    params: ResultOfSubscribeCollection,
) -> ApiResult<()> {
    if let Some(mut sender) = extract_subscription_handle(&params.handle).await {
        let _ = sender.send(true);
    }

    Ok(())
}

/// Network access.
#[derive(ApiModule)]
#[api_module(name = "net")]
pub(crate) struct NetModule;

impl ModuleReg for NetModule {
    fn reg(reg: &mut Registrar) {
        reg.async_f(query_collection, query_collection_api);
        reg.async_f(wait_for_collection, wait_for_collection_api);
        reg.async_f(unsubscribe, unsubscribe_api);
        reg.async_f_callback(subscribe_collection, subscribe_collection_api);
    }
}
