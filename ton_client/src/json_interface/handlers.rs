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

use crate::client::{ClientContext, Error};
use crate::error::ClientResult;
use futures::Future;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;
use std::sync::Arc;

use super::request::Request;
use super::runtime::{AsyncHandler, SyncHandler};

fn parse_params<P: DeserializeOwned>(params_json: &str) -> ClientResult<P> {
    serde_json::from_str(params_json).map_err(|err| Error::invalid_params(params_json, err))
}

pub struct SpawnHandlerCallback<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P, Arc<Request>) -> Fut + 'static,
{
    handler: Arc<F>,
    // Mutex is needed to have Sync trait implemented for struct
    phantom: PhantomData<std::sync::Mutex<(P, R, Fut)>>,
}

impl<P, R, Fut, F> SpawnHandlerCallback<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P, Arc<Request>) -> Fut + 'static,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
            phantom: PhantomData,
        }
    }
}

impl<P, R, Fut, F> AsyncHandler for SpawnHandlerCallback<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Send + Future<Output = ClientResult<R>> + 'static,
    F: Send + Sync + Fn(Arc<ClientContext>, P, Arc<Request>) -> Fut + 'static,
{
    fn handle(&self, context: Arc<ClientContext>, params_json: String, request: Request) {
        let handler = self.handler.clone();
        let context_copy = context.clone();
        context.env.spawn(async move {
            let request = Arc::new(request);
            match parse_params(&params_json) {
                Ok(params) => {
                    let result = handler(context_copy, params, request.clone()).await;
                    request.response_result(result);
                }
                Err(err) => request.finish_with_error(err),
            };
        });
    }
}

pub struct SpawnHandler<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P) -> Fut + 'static,
{
    handler: Arc<F>,
    // Mutex is needed to have Sync trait implemented for struct
    phantom: PhantomData<std::sync::Mutex<(P, R, Fut)>>,
}

impl<P, R, Fut, F> SpawnHandler<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P) -> Fut + 'static,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
            phantom: PhantomData,
        }
    }
}

impl<P, R, Fut, F> AsyncHandler for SpawnHandler<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Send + Future<Output = ClientResult<R>> + 'static,
    F: Send + Sync + Fn(Arc<ClientContext>, P) -> Fut + 'static,
{
    fn handle(&self, context: Arc<ClientContext>, params_json: String, request: Request) {
        let handler = self.handler.clone();
        let context_copy = context.clone();
        context.env.spawn(async move {
            match parse_params(&params_json) {
                Ok(params) => {
                    let result = handler(context_copy, params).await;
                    request.finish_with_result(result);
                }
                Err(err) => request.finish_with_error(err),
            };
        });
    }
}

pub struct SpawnNoArgsHandler<R, Fut, F>
where
    R: Send + Serialize + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>) -> Fut + 'static,
{
    handler: Arc<F>,
    // Mutex is needed to have Sync trait implemented for struct
    phantom: PhantomData<std::sync::Mutex<(R, Fut)>>,
}

impl<R, Fut, F> SpawnNoArgsHandler<R, Fut, F>
where
    R: Send + Serialize + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>) -> Fut + 'static,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
            phantom: PhantomData,
        }
    }
}

impl<R, Fut, F> AsyncHandler for SpawnNoArgsHandler<R, Fut, F>
where
    R: Send + Serialize + 'static,
    Fut: Send + Future<Output = ClientResult<R>> + 'static,
    F: Send + Sync + Fn(Arc<ClientContext>) -> Fut + 'static,
{
    fn handle(&self, context: Arc<ClientContext>, _params_json: String, request: Request) {
        let handler = self.handler.clone();
        let context_copy = context.clone();
        context.env.spawn(async move {
            request.finish_with_result(handler(context_copy).await);
        });
    }
}

pub struct CallHandler<P, R, F>
where
    P: Send + DeserializeOwned,
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>, P) -> ClientResult<R>,
{
    handler: F,
    phantom: PhantomData<std::sync::Mutex<(P, R)>>,
}

impl<P, R, F> CallHandler<P, R, F>
where
    P: Send + DeserializeOwned,
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>, P) -> ClientResult<R>,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler,
            phantom: PhantomData,
        }
    }
}

impl<P, R, F> SyncHandler for CallHandler<P, R, F>
where
    P: Send + DeserializeOwned,
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>, P) -> ClientResult<R>,
{
    fn handle(&self, context: Arc<ClientContext>, params_json: &str) -> ClientResult<String> {
        match parse_params(params_json) {
            Ok(params) => (self.handler)(context, params).and_then(|x| {
                serde_json::to_string(&x).map_err(|err| Error::cannot_serialize_result(err))
            }),
            Err(err) => Err(err),
        }
    }
}

pub struct CallNoArgsHandler<R, F>
where
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>) -> ClientResult<R>,
{
    handler: F,
    phantom: PhantomData<std::sync::Mutex<R>>,
}

impl<R, F> CallNoArgsHandler<R, F>
where
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>) -> ClientResult<R>,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler,
            phantom: PhantomData,
        }
    }
}

impl<R, F> SyncHandler for CallNoArgsHandler<R, F>
where
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>) -> ClientResult<R>,
{
    fn handle(&self, context: Arc<ClientContext>, _params_json: &str) -> ClientResult<String> {
        match (self.handler)(context) {
            Ok(result) => {
                serde_json::to_string(&result).map_err(|err| Error::cannot_serialize_result(err))
            }
            Err(err) => Err(err),
        }
    }
}
