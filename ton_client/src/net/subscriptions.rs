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

use super::Error;
use crate::client::ClientContext;
use crate::error::{AddNetworkUrl, ClientResult};
use futures::{Future, FutureExt, StreamExt};
use rand::RngCore;
use tokio::sync::mpsc::{channel, Sender};

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

async fn add_subscription_handle(
    context: &ClientContext,
    handle: u32,
    sender: Sender<SubscriptionAction>,
) {
    context
        .net
        .subscriptions
        .lock()
        .await
        .insert(handle, sender);
}

async fn extract_subscription_handle(
    context: &ClientContext,
    handle: &u32,
) -> Option<Sender<SubscriptionAction>> {
    context.net.subscriptions.lock().await.remove(handle)
}

async fn create_collection_subscription(
    context: std::sync::Arc<ClientContext>,
    params: &ParamsOfSubscribeCollection,
) -> ClientResult<super::server_link::Subscription> {
    let client = context.get_server_link()?;
    client
        .subscribe_collection(
            &params.collection,
            params.filter.as_ref().unwrap_or(&json!({})),
            &params.result,
        )
        .await
        .map_err(|err| Error::queries_subscribe_failed(err))
        .add_network_url(client)
        .await
}

pub async fn subscribe_collection<F: Future<Output = ()> + Send>(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfSubscribeCollection,
    callback: impl Fn(ClientResult<ResultOfSubscription>) -> F + Send + Sync + 'static,
) -> ClientResult<ResultOfSubscribeCollection> {
    let handle = rand::thread_rng().next_u32();

    let mut subscription = Some(create_collection_subscription(context.clone(), &params).await?);

    let (sender, mut receiver) = channel(10);
    add_subscription_handle(&context, handle, sender).await;

    // spawn thread which reads subscription stream and calls callback with data
    context.clone().env.spawn(Box::pin(async move {
        let subscription = subscription.take().unwrap();
        let mut data_stream = subscription.data_stream.fuse();
        let wait_action = receiver.recv().fuse();
        futures::pin_mut!(wait_action);
        loop {
            futures::select!(
                // waiting next subscription data
                data = data_stream.select_next_some() => {
                    callback(data.map(|data| ResultOfSubscription { result: data })).await
                },
                // waiting for some action with subscription (the only action is Finish)
                _action = wait_action => {
                    break;
                }
            );
        }
        subscription.unsubscribe.await;
    }));

    Ok(ResultOfSubscribeCollection { handle })
}

async fn create_subscription(
    context: std::sync::Arc<ClientContext>,
    params: &ParamsOfSubscribe,
) -> ClientResult<super::server_link::Subscription> {
    let client = context.get_server_link()?;
    client
        .subscribe(params.subscription.clone(), params.variables.clone())
        .await
        .map_err(|err| Error::queries_subscribe_failed(err))
        .add_network_url(client)
        .await
}

pub async fn subscribe<F: Future<Output = ()> + Send>(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfSubscribe,
    callback: impl Fn(ClientResult<ResultOfSubscription>) -> F + Send + Sync + 'static,
) -> ClientResult<ResultOfSubscribeCollection> {
    let handle = rand::thread_rng().next_u32();

    let mut subscription = Some(create_subscription(context.clone(), &params).await?);

    let (sender, mut receiver) = channel(10);
    add_subscription_handle(&context, handle, sender).await;

    // spawn thread which reads subscription stream and calls callback with data
    context.clone().env.spawn(Box::pin(async move {
        let subscription = subscription.take().unwrap();
        let mut data_stream = subscription.data_stream.fuse();
        let wait_action = receiver.recv().fuse();
        futures::pin_mut!(wait_action);
        loop {
            futures::select!(
                // waiting next subscription data
                data = data_stream.select_next_some() => {
                    callback(data.map(|data| ResultOfSubscription { result: data })).await
                },
                // waiting for some action with subscription (the only action is Finish)
                _action = wait_action => {
                    break;
                }
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
    context: std::sync::Arc<ClientContext>,
    params: ResultOfSubscribeCollection,
) -> ClientResult<()> {
    if let Some(sender) = extract_subscription_handle(&context, &params.handle).await {
        let _ = sender.send(SubscriptionAction::Finish).await;
    }
    Ok(())
}
