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
use crate::error::{ApiError, ApiResult};
use futures::{Stream, StreamExt};
use rand::RngCore;
use std::collections::HashMap;
use std::sync::Mutex;

#[cfg(test)]
mod tests;

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
pub(crate) struct ParamsOfQueryCollection {
    /// collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// collection filter
    pub filter: Option<serde_json::Value>,
    /// projection (result) string
    pub result: String,
    /// sorting order
    pub order: Option<Vec<ton_sdk::OrderBy>>,
    /// number of documents to return
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
pub(crate) struct ResultOfQueryCollection {
    /// objects that match provided criteria
    pub result: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
pub(crate) struct ParamsOfWaitForCollection {
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
pub(crate) struct ResultOfWaitForCollection {
    /// first found object that match provided criteria
    pub result: serde_json::Value,
}

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
pub(crate) struct ParamsOfSubscribeCollection {
    /// collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// collection filter
    pub filter: Option<serde_json::Value>,
    /// projection (result) string
    pub result: String,
}

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
pub(crate) struct ResultOfSubscribeCollection {
    /// handle to subscription. It then can be used in `get_next_subscription_data` function
    /// and must be closed with `unsubscribe`
    pub handle: u32,
}

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
pub(crate) struct ResultOfGetNextSubscriptionData {
    /// first appeared object that match provided criteria
    pub result: serde_json::Value,
}

lazy_static! {
    static ref STREAMS: Mutex<
        HashMap<
            u32,
            Box<dyn Stream<Item = Result<serde_json::Value, failure::Error>> + Send + Unpin>,
        >,
    > = Mutex::new(HashMap::new());
}

fn add_handle(
    handle: u32,
    stream: Box<dyn Stream<Item = Result<serde_json::Value, failure::Error>> + Send + Unpin>,
) {
    STREAMS.lock().unwrap().insert(handle, stream);
}

fn extract_handle(
    handle: &u32,
) -> Option<Box<dyn Stream<Item = Result<serde_json::Value, failure::Error>> + Send + Unpin>> {
    STREAMS.lock().unwrap().remove(handle)
}

pub(crate) async fn query_collection(
    context: &mut ClientContext,
    params: ParamsOfQueryCollection,
) -> ApiResult<ResultOfQueryCollection> {
    let client = context.get_client()?;
    let result = client
        .query(
            &params.collection,
            &params.filter.unwrap_or(json!({})).to_string(),
            &params.result,
            params.order,
            params.limit,
            None,
        )
        .await
        .map_err(|err| {
            crate::error::apierror_from_sdkerror(&err, ApiError::queries_query_failed, Some(client))
        })?;

    let result = serde_json::from_value(result)
        .map_err(|err| ApiError::queries_query_failed(format!("Can not parse result: {}", err)))?;

    Ok(ResultOfQueryCollection { result })
}

pub(crate) async fn wait_for_collection(
    context: &mut ClientContext,
    params: ParamsOfWaitForCollection,
) -> ApiResult<ResultOfWaitForCollection> {
    let client = context.get_client()?;
    let result = client
        .wait_for(
            &params.collection,
            &params.filter.unwrap_or(json!({})).to_string(),
            &params.result,
            params.timeout,
        )
        .await
        .map_err(|err| {
            crate::error::apierror_from_sdkerror(
                &err,
                ApiError::queries_wait_for_failed,
                Some(client),
            )
        })?;

    Ok(ResultOfWaitForCollection { result })
}

pub(crate) fn subscribe_collection(
    context: &mut ClientContext,
    params: ParamsOfSubscribeCollection,
) -> ApiResult<ResultOfSubscribeCollection> {
    let client = context.get_client()?;
    let stream = client
        .subscribe(
            &params.collection,
            &params.filter.unwrap_or(json!({})).to_string(),
            &params.result,
        )
        .map_err(|err| ApiError::queries_subscribe_failed(err).add_network_url(client))?;

    let handle = rand::thread_rng().next_u32();

    add_handle(handle, Box::new(stream));

    Ok(ResultOfSubscribeCollection { handle })
}

pub(crate) async fn get_next_subscription_data(
    context: &mut ClientContext,
    params: ResultOfSubscribeCollection,
) -> ApiResult<ResultOfGetNextSubscriptionData> {
    let mut stream = extract_handle(&params.handle)
        .ok_or(ApiError::queries_get_next_failed("Invalid handle"))?;

    let result = stream
        .by_ref()
        .next()
        .await
        .ok_or(ApiError::queries_get_next_failed("None value"))?
        .map_err(|err| {
            crate::error::apierror_from_sdkerror(
                &err,
                ApiError::queries_get_next_failed,
                context.get_client().ok(),
            )
        })?;

    add_handle(params.handle, stream);

    Ok(ResultOfGetNextSubscriptionData { result })
}

pub(crate) fn unsubscribe(
    _context: &mut ClientContext,
    params: ResultOfSubscribeCollection,
) -> ApiResult<()> {
    let _stream = extract_handle(&params.handle)
        .ok_or(ApiError::queries_get_next_failed("Invalid handle"))?;

    Ok(())
}

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.spawn(
        "queries.query_collection",
        |context: &mut crate::client::ClientContext, params: ParamsOfQueryCollection| {
            let mut runtime = context.take_runtime()?;
            let result = runtime.block_on(query_collection(context, params));
            context.runtime = Some(runtime);
            result
        },
    );
    handlers.spawn(
        "queries.wait_for_collection",
        |context: &mut crate::client::ClientContext, params: ParamsOfWaitForCollection| {
            let mut runtime = context.take_runtime()?;
            let result = runtime.block_on(wait_for_collection(context, params));
            context.runtime = Some(runtime);
            result
        },
    );
    handlers.spawn("queries.subscribe_collection", subscribe_collection);
    handlers.spawn(
        "queries.get_next_subscription_data",
        |context: &mut crate::client::ClientContext, params: ResultOfSubscribeCollection| {
            let mut runtime = context.take_runtime()?;
            let result = runtime.block_on(get_next_subscription_data(context, params));
            context.runtime = Some(runtime);
            result
        },
    );
    handlers.spawn("queries.unsubscribe", unsubscribe);
}
