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

use crate::client::{parse_params, ClientContext, Error};
use crate::error::{ApiError, ApiResult};
use api_info::{ApiModule, ApiType};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::api::interop::ResponseHandler;
use crate::{ResponseType, StringData};
use api_info::{Module, API};
#[cfg(feature = "node_interaction")]
use std::future::Future;

trait SyncHandler {
    fn handle(&self, context: Arc<ClientContext>, params_json: &str) -> ApiResult<String>;
}

trait AsyncHandler {
    fn handle(
        &self,
        context: Arc<ClientContext>,
        params_json: String,
        request_id: u32,
        response_handler: ResponseHandler,
    );
}

pub(crate) struct ApiDispatcher {
    pub(crate) api: API,

    sync_runners: HashMap<String, Box<dyn SyncHandler + Sync>>,
    async_runners: HashMap<String, Box<dyn AsyncHandler + Sync>>,
}

#[cfg(feature = "node_interaction")]
struct SpawnHandlerCallback<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Future<Output = ApiResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P, Arc<Request>) -> Fut + 'static,
{
    handler: Arc<F>,
    // Mutex is needed to have Sync trait implemented for struct
    phantom: PhantomData<std::sync::Mutex<(P, R, Fut)>>,
}

#[cfg(feature = "node_interaction")]
impl<P, R, Fut, F> SpawnHandlerCallback<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Future<Output = ApiResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P, Arc<Request>) -> Fut + 'static,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
            phantom: PhantomData,
        }
    }
}

#[cfg(feature = "node_interaction")]
impl<P, R, Fut, F> AsyncHandler for SpawnHandlerCallback<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Send + Future<Output = ApiResult<R>> + 'static,
    F: Send + Sync + Fn(Arc<ClientContext>, P, Arc<Request>) -> Fut + 'static,
{
    fn handle(
        &self,
        context: Arc<ClientContext>,
        params_json: String,
        request_id: u32,
        response_handler: ResponseHandler,
    ) {
        let handler = self.handler.clone();
        let context_copy = context.clone();
        context.async_runtime_handle.enter(move || {
            tokio::spawn(async move {
                let request = Arc::new(Request::new(response_handler, request_id));
                match parse_params(&params_json) {
                    Ok(params) => {
                        let result = handler(context_copy, params, request.clone()).await;
                        request.send_result(result, false);
                    }
                    Err(err) => request.finish_with_error(err),
                };
            });
        });
    }
}

#[cfg(feature = "node_interaction")]
struct SpawnHandler<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Future<Output = ApiResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P) -> Fut + 'static,
{
    handler: Arc<F>,
    // Mutex is needed to have Sync trait implemented for struct
    phantom: PhantomData<std::sync::Mutex<(P, R, Fut)>>,
}

#[cfg(feature = "node_interaction")]
impl<P, R, Fut, F> SpawnHandler<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Future<Output = ApiResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P) -> Fut + 'static,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
            phantom: PhantomData,
        }
    }
}

#[cfg(feature = "node_interaction")]
impl<P, R, Fut, F> AsyncHandler for SpawnHandler<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Send + Future<Output = ApiResult<R>> + 'static,
    F: Send + Sync + Fn(Arc<ClientContext>, P) -> Fut + 'static,
{
    fn handle(
        &self,
        context: Arc<ClientContext>,
        params_json: String,
        request_id: u32,
        response_handler: ResponseHandler,
    ) {
        let handler = self.handler.clone();
        let context_copy = context.clone();
        context.async_runtime_handle.enter(move || {
            tokio::spawn(async move {
                let request = Request::new(response_handler, request_id);
                match parse_params(&params_json) {
                    Ok(params) => {
                        let result = handler(context_copy, params).await;
                        request.finish_with(result);
                    }
                    Err(err) => request.finish_with_error(err),
                };
            });
        });
    }
}

#[cfg(feature = "node_interaction")]
struct SpawnNoArgsHandler<R, Fut, F>
where
    R: Send + Serialize + 'static,
    Fut: Future<Output = ApiResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>) -> Fut + 'static,
{
    handler: Arc<F>,
    // Mutex is needed to have Sync trait implemented for struct
    phantom: PhantomData<std::sync::Mutex<(R, Fut)>>,
}

#[cfg(feature = "node_interaction")]
impl<R, Fut, F> SpawnNoArgsHandler<R, Fut, F>
where
    R: Send + Serialize + 'static,
    Fut: Future<Output = ApiResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>) -> Fut + 'static,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
            phantom: PhantomData,
        }
    }
}

#[cfg(feature = "node_interaction")]
impl<R, Fut, F> AsyncHandler for SpawnNoArgsHandler<R, Fut, F>
where
    R: Send + Serialize + 'static,
    Fut: Send + Future<Output = ApiResult<R>> + 'static,
    F: Send + Sync + Fn(Arc<ClientContext>) -> Fut + 'static,
{
    fn handle(
        &self,
        context: Arc<ClientContext>,
        _params_json: String,
        request_id: u32,
        response_handler: ResponseHandler,
    ) {
        let handler = self.handler.clone();
        let context_copy = context.clone();
        context.async_runtime_handle.enter(move || {
            tokio::spawn(async move {
                Request::new(response_handler, request_id).finish_with(handler(context_copy).await);
            });
        });
    }
}

struct CallHandler<P, R, F>
where
    P: Send + DeserializeOwned,
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>, P) -> ApiResult<R>,
{
    handler: F,
    phantom: PhantomData<std::sync::Mutex<(P, R)>>,
}

impl<P, R, F> CallHandler<P, R, F>
where
    P: Send + DeserializeOwned,
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>, P) -> ApiResult<R>,
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
    F: Fn(Arc<ClientContext>, P) -> ApiResult<R>,
{
    fn handle(&self, context: Arc<ClientContext>, params_json: &str) -> ApiResult<String> {
        match parse_params(params_json) {
            Ok(params) => (self.handler)(context, params).and_then(|x| {
                serde_json::to_string(&x).map_err(|err| Error::cannot_serialize_result(err))
            }),
            Err(err) => Err(err),
        }
    }
}

struct CallNoArgsHandler<R, F>
where
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>) -> ApiResult<R>,
{
    handler: F,
    phantom: PhantomData<std::sync::Mutex<R>>,
}

impl<R, F> CallNoArgsHandler<R, F>
where
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>) -> ApiResult<R>,
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
    F: Fn(Arc<ClientContext>) -> ApiResult<R>,
{
    fn handle(&self, context: Arc<ClientContext>, _params_json: &str) -> ApiResult<String> {
        match (self.handler)(context) {
            Ok(result) => {
                serde_json::to_string(&result).map_err(|err| Error::cannot_serialize_result(err))
            }
            Err(err) => Err(err),
        }
    }
}

impl ApiDispatcher {
    pub fn new() -> ApiDispatcher {
        ApiDispatcher {
            api: API {
                version: "1.0.0".into(),
                modules: vec![],
            },
            sync_runners: HashMap::new(),
            #[cfg(feature = "node_interaction")]
            async_runners: HashMap::new(),
        }
    }

    pub fn sync_dispatch(
        &self,
        context: Arc<ClientContext>,
        function_name: String,
        params_json: String,
    ) -> ApiResult<String> {
        match self.sync_runners.get(&function_name) {
            Some(handler) => handler.handle(context, params_json.as_str()),
            None => Err(ApiError::unknown_function(&function_name)),
        }
    }

    #[cfg(feature = "node_interaction")]
    pub fn async_dispatch(
        &self,
        context: Arc<ClientContext>,
        function_name: String,
        params_json: String,
        request_id: u32,
        response_handler: ResponseHandler,
    ) {
        match self.async_runners.get(&function_name) {
            Some(handler) => handler.handle(context, params_json, request_id, response_handler),
            None => Request::new(response_handler, request_id)
                .finish_with_error(ApiError::unknown_function(&function_name)),
        }
    }

    #[cfg(not(feature = "node_interaction"))]
    pub fn async_dispatch(
        &self,
        context: Arc<ClientContext>,
        function_name: String,
        params_json: String,
        request_id: u32,
        response_handler: ResponseHandler,
    ) {
        Request::new(response_handler, request_id).finish_with(self.sync_dispatch(
            context,
            function_name,
            params_json,
        ));
    }

    pub(crate) fn register<'h, M: ApiModule + ModuleReg>(&'h mut self) {
        let mut registrar = Registrar::<'h> {
            dispatcher: self,
            module: M::api(),
        };
        M::reg(&mut registrar);
        registrar.dispatcher.api.modules.push(registrar.module);
    }
}

pub(crate) struct Registrar<'a> {
    module: Module,
    dispatcher: &'a mut ApiDispatcher,
}

impl Registrar<'_> {
    pub fn api_type<T: ApiType>(&mut self) {
        self.module.types.push(T::api());
    }

    pub fn api_async_fn<P, R, F>(
        &mut self,
        handler: fn(context: std::sync::Arc<ClientContext>, params: P) -> F,
        api: fn() -> api_info::Function,
    ) where
        P: ApiType + Send + DeserializeOwned + 'static,
        R: ApiType + Send + Serialize + 'static,
        F: Send + Future<Output = ApiResult<R>> + 'static,
    {
        self.api_type::<P>();
        self.api_type::<R>();
        let function = api();
        let name = format!("{}.{}", self.module.name, function.name);
        self.module.functions.push(function);
        self.dispatcher
            .async_runners
            .insert(name.clone(), Box::new(SpawnHandler::new(handler)));

        self.dispatcher.sync_runners.insert(
            name,
            Box::new(CallHandler::new(move |context, params| {
                context
                    .clone()
                    .async_runtime_handle
                    .block_on(handler(context, params))
            })),
        );
    }

    pub fn api_async_fn_with_callback<P, R, F>(
        &mut self,
        handler: fn(context: std::sync::Arc<ClientContext>, params: P, callback: Arc<Request>) -> F,
        api: fn() -> api_info::Function,
    ) where
        P: ApiType + Send + DeserializeOwned + 'static,
        R: ApiType + Send + Serialize + 'static,
        F: Send + Future<Output = ApiResult<R>> + 'static,
    {
        self.api_type::<P>();
        self.api_type::<R>();
        let function = api();
        let name = format!("{}.{}", self.module.name, function.name);
        self.module.functions.push(function);
        self.dispatcher
            .async_runners
            .insert(name.clone(), Box::new(SpawnHandlerCallback::new(handler)));
    }

    pub fn api_sync_fn<P, R>(
        &mut self,
        handler: fn(context: std::sync::Arc<ClientContext>, params: P) -> ApiResult<R>,
        api: fn() -> api_info::Function,
    ) where
        P: ApiType + Send + DeserializeOwned + 'static,
        R: ApiType + Send + Serialize + 'static,
    {
        self.api_type::<P>();
        self.api_type::<R>();
        let function = api();
        let name = format!("{}.{}", self.module.name, function.name);
        self.module.functions.push(function);

        self.dispatcher
            .sync_runners
            .insert(name.clone(), Box::new(CallHandler::new(handler)));

        #[cfg(feature = "node_interaction")]
        self.dispatcher.async_runners.insert(
            name.clone(),
            Box::new(SpawnHandler::new(move |context, params| async move {
                handler(context, params)
            })),
        );
    }

    pub fn api_sync_fn_without_args<R>(
        &mut self,
        handler: fn(context: std::sync::Arc<ClientContext>) -> ApiResult<R>,
        api: fn() -> api_info::Function,
    ) where
        R: ApiType + Send + Serialize + 'static,
    {
        self.api_type::<R>();
        let function = api();
        let name = format!("{}.{}", self.module.name, function.name);
        self.module.functions.push(function);
        self.dispatcher
            .sync_runners
            .insert(name.clone(), Box::new(CallNoArgsHandler::new(handler)));

        #[cfg(feature = "node_interaction")]
        self.dispatcher.async_runners.insert(
            name.clone(),
            Box::new(SpawnNoArgsHandler::new(move |context| async move {
                handler(context)
            })),
        );
    }
}

pub(crate) trait ModuleReg {
    fn reg(registrar: &mut Registrar);
}

pub struct Request {
    response_handler: ResponseHandler,
    request_id: u32,
}

impl Request {
    pub fn new(response_handler: ResponseHandler, request_id: u32) -> Self {
        Self {
            response_handler,
            request_id,
        }
    }

    fn call_response_handler(
        &self,
        params_json: impl Serialize,
        response_type: u32,
        finished: bool,
    ) {
        match serde_json::to_string(&params_json) {
            Ok(result) => (self.response_handler)(
                self.request_id,
                StringData::from(&result),
                response_type,
                finished,
            ),
            Err(_) => (self.response_handler)(
                self.request_id,
                StringData::from(crate::client::errors::CANNOT_SERIALIZE_RESULT),
                response_type,
                false,
            ),
        };
    }

    pub fn send_result(&self, result: ApiResult<impl Serialize>, finished: bool) {
        match result {
            Ok(result) => {
                self.call_response_handler(result, ResponseType::Success as u32, finished)
            }
            Err(err) => self.call_response_handler(err, ResponseType::Error as u32, finished),
        }
    }

    pub fn finish_with(&self, result: ApiResult<impl Serialize>) {
        self.send_result(result, true);
    }

    pub fn finish_with_error(&self, error: ApiError) {
        self.call_response_handler(error, ResponseType::Error as u32, true);
    }

    pub fn send_response(&self, result: impl Serialize, response_type: u32) {
        self.call_response_handler(result, response_type, false);
    }
}

impl Drop for Request {
    fn drop(&mut self) {
        (self.response_handler)(self.request_id, "".into(), ResponseType::Nop as u32, true)
    }
}
