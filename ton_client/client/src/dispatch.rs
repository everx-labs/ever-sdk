/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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

use crate::types::{ApiError, ApiResult};
use crate::client::ClientContext;
use super::{JsonResponse};
use std::collections::HashMap;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[cfg(feature = "node_interaction")]
use futures::Future;


impl JsonResponse {
    pub(crate) fn from_result(result_json: String) -> Self {
        Self {
            result_json,
            error_json: String::new(),
        }
    }

    pub(crate) fn from_error(err: ApiError) -> Self {
        JsonResponse {
            result_json: String::new(),
            error_json: serde_json::to_string(&err)
                .unwrap_or(r#"{"category": "sdk", "code": 1, "message": ""}"#.to_string()),
        }
    }
}

trait SyncHandler {
    fn handle(&self, context: &mut ClientContext, params_json: &str) -> JsonResponse;
}


pub(crate) struct DispatchTable {
    sync_runners: HashMap<String, Box<dyn SyncHandler + Sync>>
}

fn parse_params<P: DeserializeOwned>(params_json: &str) -> ApiResult<P> {
    serde_json::from_str(params_json).map_err(|err| ApiError::invalid_params(params_json, err))
}

struct CallHandler<P: Send + DeserializeOwned, R: Send + Serialize> {
    handler: fn(context: &mut ClientContext, params: P) -> ApiResult<R>,
}

impl<P: Send + DeserializeOwned, R: Send + Serialize> SyncHandler for CallHandler<P, R> {
    fn handle(&self, context: &mut ClientContext, params_json: &str) -> JsonResponse {
        match parse_params(params_json) {
            Ok(params) => {
                let handler = self.handler;
                let result = handler(context, params);
                match result {
                    Ok(result) =>
                        JsonResponse::from_result(serde_json::to_string(&result).unwrap()),
                    Err(err) =>
                        JsonResponse::from_error(err)
                }
            }
            Err(err) => JsonResponse::from_error(err)
        }
    }
}

#[cfg(feature = "node_interaction")]
struct AsyncCallHandler<P, R, Fut>
where
    P: Send + DeserializeOwned,
    R: Send + Serialize,
    Fut: Send + Future<Output=ApiResult<R>>
{
    handler: fn(context: &mut ClientContext, params: P) -> Fut,
}

#[cfg(feature = "node_interaction")]
impl<P, R, Fut> SyncHandler for AsyncCallHandler<P, R, Fut>
where
    P: Send + DeserializeOwned,
    R: Send + Serialize,
    Fut: Send + Future<Output=ApiResult<R>>
{
    fn handle(&self, context: &mut ClientContext, params_json: &str) -> JsonResponse {
        match parse_params(params_json) {
            Ok(params) => {
                let handler = self.handler;
                let result = run_in_runtime(handler(context, params));
                match result {
                    Ok(result) =>
                        JsonResponse::from_result(serde_json::to_string(&result).unwrap()),
                    Err(err) =>
                        JsonResponse::from_error(err)
                }
            }
            Err(err) => JsonResponse::from_error(err)
        }
    }
}

struct CallNoArgsHandler<R: Send + Serialize> {
    handler: fn(context: &mut ClientContext) -> ApiResult<R>,
}

impl<R: Send + Serialize> SyncHandler for CallNoArgsHandler<R> {
    fn handle(&self, context: &mut ClientContext, _params_json: &str) -> JsonResponse {
        let handler = self.handler;
        let result = handler(context);
        match result {
            Ok(result) =>
                JsonResponse::from_result(serde_json::to_string(&result).unwrap()),
            Err(err) =>
                JsonResponse::from_error(err)
        }
    }
}

#[cfg(feature = "node_interaction")]
pub(crate) fn run_in_runtime<R, F>(future: F) -> ApiResult<R>
where F: Future<Output=ApiResult<R>>
{
    let runtime = tokio::runtime::Builder::new()
            .basic_scheduler()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|err| ApiError::cannot_create_runtime(err))?;

    runtime.block_on(future)
}

impl DispatchTable {
    pub fn new() -> DispatchTable {
        DispatchTable {
            sync_runners: HashMap::new(),
        }
    }

    pub fn spawn<P, R>(&mut self, method: &str, handler: fn(context: &mut ClientContext, params: P) -> ApiResult<R>)
        where P: Send + DeserializeOwned + 'static, R: Send + Serialize + 'static
    {
        self.sync_runners.insert(method.to_string(), Box::new(CallHandler { handler }));
    }

    #[cfg(feature = "node_interaction")]
    pub fn spawn_async<P, R, Fut>(&mut self, method: &str, handler: fn(context: &mut ClientContext, params: P) -> Fut)
    where
        P: Send + DeserializeOwned + 'static,
        R: Send + Serialize + 'static,
        Fut: Send + Future<Output=ApiResult<R>> + 'static
    {
        self.sync_runners.insert(method.to_string(), Box::new(AsyncCallHandler { handler }));
    }

//    pub fn spawn_no_args<R>(&mut self, method: &str, handler: fn(context: &mut ClientContext) -> ApiResult<R>)
//        where R: Send + Serialize + 'static
//    {
//        self.sync_runners.insert(method.to_string(), Box::new(CallNoArgsHandler { handler }));
//    }

    pub fn call<P, R>(&mut self, method: &str, handler: fn(context: &mut ClientContext, params: P) -> ApiResult<R>)
        where P: Send + DeserializeOwned + 'static, R: Send + Serialize + 'static
    {
        self.sync_runners.insert(method.to_string(), Box::new(CallHandler { handler }));
    }

    pub fn call_no_args<R>(&mut self, method: &str, handler: fn(context: &mut ClientContext) -> ApiResult<R>)
        where R: Send + Serialize + 'static
    {
        self.sync_runners.insert(method.to_string(), Box::new(CallNoArgsHandler { handler }));
    }

    pub fn sync_dispatch(&self, context: &mut ClientContext, method: String, params_json: String) -> JsonResponse {
        match self.sync_runners.get(&method) {
            Some(handler) => handler.handle(context, params_json.as_str()),
            None => JsonResponse::from_error(ApiError::unknown_method(&method))
        }
    }
}
