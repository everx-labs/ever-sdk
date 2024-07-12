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
 *
 */

use super::request::Request;
use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::net::{ParamsOfSubscribeCollection, ResultOfSubscribeCollection, ResultOfSubscription};
use crate::net::subscriptions::ParamsOfSubscribe;

/// Creates a collection subscription
///
/// Triggers for each insert/update of data that satisfies
/// the `filter` conditions.
/// The projection fields are limited to `result` fields.
///
/// The subscription is a persistent communication channel between
/// client and Free TON Network.
/// All changes in the blockchain will be reflected in realtime.
/// Changes means inserts and updates of the blockchain entities.
///
/// ### Important Notes on Subscriptions
///
/// Unfortunately sometimes the connection with the network brakes down.
/// In this situation the library attempts to reconnect to the network.
/// This reconnection sequence can take significant time.
/// All of this time the client is disconnected from the network.
///
/// Bad news is that all blockchain changes that happened while
/// the client was disconnected are lost.
///
/// Good news is that the client report errors to the callback when
/// it loses and resumes connection.
///
/// So, if the lost changes are important to the application then
/// the application must handle these error reports.
///
/// Library reports errors with `responseType` == 101
/// and the error object passed via `params`.
///
/// When the library has successfully reconnected
/// the application receives callback with
/// `responseType` == 101 and `params.code` == 614 (NetworkModuleResumed).
///
/// Application can use several ways to handle this situation:
/// - If application monitors changes for the single blockchain
/// object (for example specific account):  application
/// can perform a query for this object and handle actual data as a
/// regular data from the subscription.
/// - If application monitors sequence of some blockchain objects
/// (for example transactions of the specific account): application must
/// refresh all cached (or visible to user) lists where this sequences presents.
///
#[api_function]
pub(crate) async fn subscribe_collection(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfSubscribeCollection,
    callback: std::sync::Arc<Request>,
) -> ClientResult<ResultOfSubscribeCollection> {
    let callback = move |result: ClientResult<ResultOfSubscription>| {
        match result {
            Ok(result) => {
                callback.response(result, crate::net::SubscriptionResponseType::Ok as u32)
            }
            Err(err) => callback.response(err, crate::net::SubscriptionResponseType::Error as u32),
        }
        futures::future::ready(())
    };

    crate::net::subscribe_collection(context, params, callback).await
}

/// Creates a subscription
///
/// The subscription is a persistent communication channel between
/// client and Everscale Network.
///
/// ### Important Notes on Subscriptions
///
/// Unfortunately sometimes the connection with the network breaks down.
/// In this situation the library attempts to reconnect to the network.
/// This reconnection sequence can take significant time.
/// All of this time the client is disconnected from the network.
///
/// Bad news is that all changes that happened while
/// the client was disconnected are lost.
///
/// Good news is that the client report errors to the callback when
/// it loses and resumes connection.
///
/// So, if the lost changes are important to the application then
/// the application must handle these error reports.
///
/// Library reports errors with `responseType` == 101
/// and the error object passed via `params`.
///
/// When the library has successfully reconnected
/// the application receives callback with
/// `responseType` == 101 and `params.code` == 614 (NetworkModuleResumed).
///
/// Application can use several ways to handle this situation:
/// - If application monitors changes for the single
/// object (for example specific account):  application
/// can perform a query for this object and handle actual data as a
/// regular data from the subscription.
/// - If application monitors sequence of some objects
/// (for example transactions of the specific account): application must
/// refresh all cached (or visible to user) lists where this sequences presents.
///
#[api_function]
pub(crate) async fn subscribe(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfSubscribe,
    callback: std::sync::Arc<Request>,
) -> ClientResult<ResultOfSubscribeCollection> {
    let callback = move |result: ClientResult<ResultOfSubscription>| {
        match result {
            Ok(result) => {
                callback.response(result, crate::net::SubscriptionResponseType::Ok as u32)
            }
            Err(err) => callback.response(err, crate::net::SubscriptionResponseType::Error as u32),
        }
        futures::future::ready(())
    };

    crate::net::subscribe(context, params, callback).await
}
