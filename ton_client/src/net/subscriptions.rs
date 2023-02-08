/*
* Copyright 2018-2021 TON Labs LTD.
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
use futures::Future;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, num_derive::FromPrimitive)]
pub enum SubscriptionResponseType {
    Ok = 100,
    Error = 101,
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfSubscribeCollection {
    /// Collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// Collection filter
    pub filter: Option<serde_json::Value>,
    /// Projection (result) string
    pub result: String,
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfSubscribe {
    /// GraphQL subscription text.
    pub subscription: String,
    /// Variables used in subscription. Must be a map with named values that
    /// can be used in query.
    pub variables: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ResultOfSubscribeCollection {
    /// Subscription handle. Must be closed with `unsubscribe`
    pub handle: u32,
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone, Debug)]
pub struct ResultOfSubscription {
    /// First appeared object that matches the provided criteria
    pub result: serde_json::Value,
}

#[derive(PartialEq, Debug)]
pub(crate) enum SubscriptionAction {
    Finish,
}

pub async fn subscribe_collection<F: Future<Output = ()> + Send>(
    context: Arc<ClientContext>,
    params: ParamsOfSubscribeCollection,
    callback: impl Fn(ClientResult<ResultOfSubscription>) -> F + Send + Sync + 'static,
) -> ClientResult<ResultOfSubscribeCollection> {
    context
        .net
        .subscribe_collection(params.collection, params.filter, params.result, callback)
        .await
        .map(|handle| ResultOfSubscribeCollection { handle })
}

pub async fn subscribe<F: Future<Output = ()> + Send>(
    context: Arc<ClientContext>,
    params: ParamsOfSubscribe,
    callback: impl Fn(ClientResult<ResultOfSubscription>) -> F + Send + Sync + 'static,
) -> ClientResult<ResultOfSubscribeCollection> {
    context
        .net
        .subscribe(params.subscription, params.variables, callback)
        .await
        .map(|handle| ResultOfSubscribeCollection { handle })
}

/// Cancels a subscription
///
/// Cancels a subscription specified by its handle.
#[api_function]
pub async fn unsubscribe(
    context: Arc<ClientContext>,
    params: ResultOfSubscribeCollection,
) -> ClientResult<()> {
    context.net.unsubscribe(params.handle).await
}
