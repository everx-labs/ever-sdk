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

use futures::{Stream, StreamExt};
use std::collections::HashMap;
use std::sync::Mutex;
use rand::RngCore;

use crate::client::ClientContext;
use crate::types::{ApiResult, ApiError};

#[derive(Serialize, Deserialize)]
pub(crate) struct ParamsOfQuery {
    pub table: String,
    pub filter: String,
    pub result: String,
    pub order: Option<ton_sdk::OrderBy>,
    pub limit: Option<u32>
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ParamsOfSubscribe {
    pub table: String,
    pub filter: String,
    pub result: String
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ParamsOfWaitFor {
    pub table: String,
    pub filter: String,
    pub result: String,
    pub timeout: Option<u32>
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfQuery {
    pub result: serde_json::Value
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SubscribeHandle {
    pub handle: StreamHandle
}

type StreamHandle = u32;

lazy_static! {
    static ref STREAMS: 
        Mutex<HashMap<StreamHandle, Box<dyn Stream<Item=Result<serde_json::Value, failure::Error>> + Send + Unpin>>>
            = Mutex::new(HashMap::new());
}

pub(crate) async fn query(context: &mut ClientContext, params: ParamsOfQuery) -> ApiResult<ResultOfQuery> {
    let client = context.get_client()?;
    let result = client.query(&params.table, &params.filter, &params.result, params.order, params.limit, Some(0))
        .await
        .map_err(|err| crate::types::apierror_from_sdkerror(err, ApiError::queries_query_failed))?;

    Ok(ResultOfQuery{ result })
}

pub(crate) async fn wait_for(context: &mut ClientContext, params: ParamsOfWaitFor) -> ApiResult<ResultOfQuery> {
    let client = context.get_client()?;
    let result = client.wait_for(&params.table, &params.filter, &params.result, params.timeout)
        .await
        .map_err(|err| crate::types::apierror_from_sdkerror(err, ApiError::queries_wait_for_failed))?;

    Ok(ResultOfQuery{ result })
}

pub(crate) fn subscribe(context: &mut ClientContext, params: ParamsOfSubscribe) -> ApiResult<SubscribeHandle> {
    let client = context.get_client()?;
    let stream = client.subscribe(&params.table, &params.filter, &params.result)
        .map_err(|err| ApiError::queries_subscribe_failed(err))?;

    let handle =  rand::thread_rng().next_u32();

    add_handle(handle, Box::new(stream));

    Ok(SubscribeHandle{ handle })
}

pub(crate) async fn get_next(_context: &mut ClientContext, params: SubscribeHandle) -> ApiResult<ResultOfQuery> {
    let mut stream = extract_handle(&params.handle)
        .ok_or(ApiError::queries_get_next_failed("Invalid handle"))?;

    let result = stream.by_ref()
        .next()
        .await
        .ok_or(ApiError::queries_get_next_failed("None value"))?
        .map_err(|err| crate::types::apierror_from_sdkerror(err, ApiError::queries_get_next_failed))?;

    add_handle(params.handle, stream);

    Ok(ResultOfQuery{ result: result })
}

pub(crate) fn unsubscribe(_context: &mut ClientContext, params: SubscribeHandle) -> ApiResult<()> {
    let _stream = extract_handle(&params.handle)
        .ok_or(ApiError::queries_get_next_failed("Invalid handle"))?;

    Ok(())
}

fn add_handle(handle: StreamHandle, stream:  Box<dyn Stream<Item=Result<serde_json::Value, failure::Error>> + Send + Unpin>) {
    STREAMS.lock().unwrap().insert(handle, stream);
}

fn extract_handle(handle: &StreamHandle) -> Option< Box<dyn Stream<Item=Result<serde_json::Value, failure::Error>> + Send + Unpin>> {
    STREAMS.lock().unwrap().remove(handle)
}

