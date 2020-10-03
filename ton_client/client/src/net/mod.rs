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
use crate::dispatch::DispatchTable;
use crate::error::ApiResult;
use crate::interop::JsonResponse;
use futures::{FutureExt, StreamExt};
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

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
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

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
pub struct ResultOfQueryCollection {
    /// objects that match provided criteria
    pub result: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
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

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
pub struct ResultOfWaitForCollection {
    /// first found object that match provided criteria
    pub result: serde_json::Value,
}

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
pub struct ParamsOfSubscribeCollection {
    /// collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// collection filter
    pub filter: Option<serde_json::Value>,
    /// projection (result) string
    pub result: String,
    /// registered callback ID to receive subscription data
    pub callback_id: u32,
}

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
pub struct ResultOfSubscribeCollection {
    /// handle to subscription. It then can be used in `get_next_subscription_data` function
    /// and must be closed with `unsubscribe`
    pub handle: u32,
}

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
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

#[function_info]
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

#[function_info]
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

#[function_info]
pub async fn subscribe_collection(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfSubscribeCollection,
) -> ApiResult<ResultOfSubscribeCollection> {
    let callback_id = params.callback_id;
    let callback = context.get_callback(callback_id)?;

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
    let context_copy = context.clone();
    context.env.spawn(Box::pin(async move {
        let wait_abortion = receiver.recv().fuse();
        futures::pin_mut!(wait_abortion);
        let mut data_stream = subscription.data_stream.fuse();
        loop {
            futures::select!(
                // waiting next subscription data
                data = data_stream.select_next_some() => {
                    match data {
                        Ok(data) => {
                            let result = ResultOfSubscription {
                                result: data
                            };
                            JsonResponse::from_result(serde_json::to_string(&result).unwrap())
                                .send(&*callback, callback_id, 0);
                        }
                        Err(err) => {
                            JsonResponse::from_error(err.add_network_url(context_copy.get_client().unwrap()))
                                .send(&*callback, callback_id, 0);
                        }
                    }
                },
                // waiting for unsubscribe
                _ = wait_abortion => break
            );
        }
        subscription.unsubscribe.await;
    }));

    Ok(ResultOfSubscribeCollection { handle })
}

#[function_info]
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
#[derive(TypeInfo)]
#[type_info(name = "net")]
struct NetModule;

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.register_module::<NetModule>(&[], |reg| {
        reg.async_func(query_collection, query_collection_info);
        reg.async_func(wait_for_collection, wait_for_collection_info);
        reg.async_func(subscribe_collection, subscribe_collection_info);
        reg.async_func(unsubscribe, unsubscribe_info);
    });
}
