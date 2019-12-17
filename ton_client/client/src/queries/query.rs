/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use futures::Stream;
use std::collections::HashMap;
use std::sync::Mutex;
use rand::RngCore;

use ton_sdk::queries_helper;
use ton_sdk::SdkError;
use client::ClientContext;
use types::{ApiResult, ApiError};

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfQuery {
    pub table: String,
    pub filter: String,
    pub result: String,
    pub order: Option<queries_helper::OrderBy>,
    pub limit: Option<u32>
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfSubscribe {
    pub table: String,
    pub filter: String,
    pub result: String
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfQuery {
    pub result: serde_json::Value
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct SubscribeHandle {
    pub handle: StreamHandle
}

type StreamHandle = u32;

lazy_static! {
    static ref STREAMS: 
        Mutex<HashMap<StreamHandle, Box<dyn Stream<Item=serde_json::Value, Error=SdkError> + Send>>>
            = Mutex::new(HashMap::new());
}

pub(crate) fn query(_context: &mut ClientContext, params: ParamsOfQuery) -> ApiResult<ResultOfQuery> {
    let stream = queries_helper::query(&params.table, &params.filter, &params.result, params.order, params.limit)
        .map_err(|err| ApiError::queries_query_failed(err))?;

    let result = stream
        .wait()
        .next()
        .ok_or(ApiError::queries_query_failed("None value"))?
        .map_err(|err| ApiError::queries_query_failed(err))?;

    Ok(ResultOfQuery{ result: result })
}

pub(crate) fn wait_for(_context: &mut ClientContext, params: ParamsOfSubscribe) -> ApiResult<ResultOfQuery> {
    let result = queries_helper::wait_for(&params.table, &params.filter, &params.result)
        .map_err(|err| ApiError::queries_wait_for_failed(err))?;

    Ok(ResultOfQuery{ result: result })
}

pub(crate) fn subscribe(_context: &mut ClientContext, params: ParamsOfSubscribe) -> ApiResult<SubscribeHandle> {
    let stream = queries_helper::subscribe(&params.table, &params.filter, &params.result)
        .map_err(|err| ApiError::queries_subscribe_failed(err))?;

    let mut rng = rand::rngs::OsRng::new()
        .map_err(|err| ApiError::queries_subscribe_failed(err))?;
    let handle =  rng.next_u32();

    add_handle(handle, stream);

    Ok(SubscribeHandle{ handle })
}

pub(crate) fn get_next(_context: &mut ClientContext, params: SubscribeHandle) -> ApiResult<ResultOfQuery> {
    let mut stream = extract_handle(&params.handle)
        .ok_or(ApiError::queries_get_next_failed("Invalid handle"))?;

    let result = stream.by_ref()
        .wait()
        .next()
        .ok_or(ApiError::queries_get_next_failed("None value"))?
        .map_err(|err| ApiError::queries_get_next_failed(err))?;

    add_handle(params.handle, stream);

    Ok(ResultOfQuery{ result: result })
}

pub(crate) fn unsubscribe(_context: &mut ClientContext, params: SubscribeHandle) -> ApiResult<()> {
    let _stream = extract_handle(&params.handle)
        .ok_or(ApiError::queries_get_next_failed("Invalid handle"))?;

    Ok(())
}

fn add_handle(handle: StreamHandle, stream: Box<dyn Stream<Item=serde_json::Value, Error=SdkError> + Send>) {
    STREAMS.lock().unwrap().insert(handle, stream);
}

fn extract_handle(handle: &StreamHandle) -> Option<Box<dyn Stream<Item=serde_json::Value, Error=SdkError> + Send>> {
    STREAMS.lock().unwrap().remove(handle)
}

