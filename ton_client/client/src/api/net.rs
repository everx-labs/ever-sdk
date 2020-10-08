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
 *
 */

use crate::client::{ClientContext};
use crate::error::ApiResult;
use crate::net::{ParamsOfSubscribeCollection, ResultOfSubscribeCollection, ResultOfSubscription};
use crate::api::request::Request;

#[api_function]
pub(crate) async fn subscribe_collection(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfSubscribeCollection,
    callback: std::sync::Arc<Request>,
) -> ApiResult<ResultOfSubscribeCollection> {
    let callback = move |result: ApiResult<ResultOfSubscription>| {
        match result {
            Ok(result) => {
                callback.send_response(result, crate::net::SubscriptionResponseType::Ok as u32)
            }
            Err(err) => {
                callback.send_response(err, crate::net::SubscriptionResponseType::Error as u32)
            }
        }
        futures::future::ready(())
    };

    crate::net::subscribe_collection(context, params, callback).await
}
