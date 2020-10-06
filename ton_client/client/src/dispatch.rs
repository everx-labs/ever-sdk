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

use super::JsonResponse;
use crate::client::{ExternalCallback, ClientContext, ResponseType};
use crate::error::{ApiError, ApiResult};
use api_info::{ApiModule, ApiType};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use api_info::{Module, API};
#[cfg(feature = "node_interaction")]
use std::future::Future;

impl JsonResponse {
    pub(crate) fn from_result(result_json: String) -> Self {
        Self {
            result_json,
            error_json: String::new(),
        }
    }

    pub(crate) fn from_api_result<R: Serialize>(result: ApiResult<R>) -> Self {
        match result {
            Ok(result) => Self {
                result_json: serde_json::to_string(&result)
                    .unwrap_or_else(|_| crate::client::Error::cannot_serialize_result()),
                error_json: String::new(),
            },
            Err(err) => Self {
                result_json: String::new(),
                error_json: serde_json::to_string(&err)
                    .unwrap_or_else(|_| crate::client::Error::cannot_serialize_error()),
            }
        }
    }

    pub(crate) fn from_error(err: ApiError) -> Self {
        JsonResponse {
            result_json: String::new(),
            error_json: serde_json::to_string(&err)
                .unwrap_or_else(|_| crate::client::Error::cannot_serialize_error()),
        }
    }
}

trait SyncHandler {
    fn handle(&self, context: Arc<ClientContext>, params_json: &str) -> JsonResponse;
}

trait AsyncHandler {
    fn handle(
        &self,
        context: Arc<ClientContext>,
        params_json: String,
        request_id: u32,
        on_result: Box<ExternalCallback>,
    );
}

pub(crate) struct DispatchTable {
    pub(crate) api: API,

    sync_runners: HashMap<String, Box<dyn SyncHandler + Sync>>,
    async_runners: HashMap<String, Box<dyn AsyncHandler + Sync>>,
}

pub(crate) fn parse_params<P: DeserializeOwned>(params_json: &str) -> ApiResult<P> {
    serde_json::from_str(params_json).map_err(|err| ApiError::invalid_params(params_json, err))
}

pub(crate) struct Callback {
    callback: Box<ExternalCallback>,
    request_id: u32,
}

impl Callback {
    pub fn new(callback: Box<ExternalCallback>, request_id: u32) -> Self {
        Self { callback, request_id }
    }

    pub fn call(&self, params_json: impl Serialize, response_type: u32) {
        match serde_json::to_string(&params_json) {
            Ok(result) => (self.callback)(self.request_id, &result, response_type, false),
            Err(_) => {
                (self.callback)(
                    self.request_id,
                    &crate::client::Error::cannot_serialize_result(),
                    ResponseType::Error as u32,
                    false)
            }
        };
    }
}

impl Drop for Callback {
    fn drop(&mut self) {
        (self.callback)(self.request_id, "", ResponseType::Nop as u32, true)
    }
}

#[cfg(feature = "node_interaction")]
struct SpawnHandlerCallback<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Future<Output = ApiResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P, Arc<Callback>) -> Fut + 'static,
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
    F: Send + Fn(Arc<ClientContext>, P, Arc<Callback>) -> Fut + 'static,
{
    pub fn new(api: Method, handler: F) -> Self {
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
    F: Send + Sync + Fn(Arc<ClientContext>, P, Arc<Callback>) -> Fut + 'static,
{
    fn handle(
        &self,
        context: Arc<ClientContext>,
        params_json: String,
        request_id: u32,
        on_result: Box<ExternalCallback>,
    ) {
        let callback = Callback::new(on_result, request_id);
        let handler = self.handler.clone();
        let context_copy = context.clone();
        context.async_runtime_handle.enter(move || {
            tokio::spawn(async move {
                let callback = Arc::new(callback);
                match parse_params(&params_json) {
                    Ok(params) => {
                        let result = handler(context_copy, params, callback.clone()).await;
                        match result {
                            Ok(result) => callback.call(result, ResponseType::Success as u32),
                            Err(err) => callback.call(err, ResponseType::Error as u32),
                        }
                    }
                    Err(err) => callback.call(err, ResponseType::Error as u32),
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
        on_result: Box<ExternalCallback>,
    ) {
        let handler = self.handler.clone();
        let context_copy = context.clone();
        context.async_runtime_handle.enter(move || {
            tokio::spawn(async move {
                let result = match parse_params(&params_json) {
                    Ok(params) => {
                        let result = handler(context_copy, params).await;
                        match result {
                            Ok(result) => {
                                JsonResponse::from_result(serde_json::to_string(&result).unwrap())
                            }
                            Err(err) => JsonResponse::from_error(err),
                        }
                    }
                    Err(err) => JsonResponse::from_error(err),
                };
                result.send(&*on_result, request_id);
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
        on_result: Box<ExternalCallback>,
    ) {
        let handler = self.handler.clone();
        let context_copy = context.clone();
        context.async_runtime_handle.enter(move || {
            tokio::spawn(async move {
                let result = handler(context_copy).await;
                let result = match result {
                    Ok(result) => {
                        JsonResponse::from_result(serde_json::to_string(&result).unwrap())
                    }
                    Err(err) => JsonResponse::from_error(err),
                };
                result.send(&*on_result, request_id);
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
    fn handle(&self, context: Arc<ClientContext>, params_json: &str) -> JsonResponse {
        match parse_params(params_json) {
            Ok(params) => {
                let result = (self.handler)(context, params);
                match result {
                    Ok(result) => {
                        JsonResponse::from_result(serde_json::to_string(&result).unwrap())
                    }
                    Err(err) => JsonResponse::from_error(err),
                }
            }
            Err(err) => JsonResponse::from_error(err),
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
    fn handle(&self, context: Arc<ClientContext>, _params_json: &str) -> JsonResponse {
        let result = (self.handler)(context);
        match result {
            Ok(result) => JsonResponse::from_result(serde_json::to_string(&result).unwrap()),
            Err(err) => JsonResponse::from_error(err),
        }
    }
}

impl DispatchTable {
    pub fn new() -> DispatchTable {
        DispatchTable {
            api: API {
                version: "1.0.0".into(),
                modules: vec![],
            },
            sync_runners: HashMap::new(),
            #[cfg(feature = "node_interaction")]
            async_runners: HashMap::new(),
        }
    }

    pub fn register_api_types(
        &mut self,
        module: &str,
        type_info_providers: Vec<fn() -> api_doc::api::Field>,
    ) {
        for type_info in type_info_providers.iter() {
            let type_info = type_info();
            let name = type_info.name.clone();
            self.api_types.insert(
                name,
                ModuleTypeInfo {
                    module: module.into(),
                    type_info,
                },
            );
        }
    }

    pub fn get_api(&self) -> api_doc::api::API {
        api_doc::api::API {
            version: "1.0.0".into(),
            methods: self
                .sync_runners
                .values()
                .map(|x| (*x.get_api()).clone())
                .collect(),
            types: Vec::new(),
        }
    }

    pub fn call<P, R>(
        &mut self,
        method: &str,
        handler: fn(context: Arc<ClientContext>, params: P) -> ApiResult<R>,
    ) where
        P: TypeInfo + Send + DeserializeOwned + 'static,
        R: TypeInfo + Send + Serialize + 'static,
    {
        let api = Method::from_types::<P, R>(method);
        self.sync_runners.insert(
            method.into(),
            Box::new(CallHandler::new(api.clone(), handler)),
        );

        #[cfg(feature = "node_interaction")]
        self.async_runners.insert(
            method.into(),
            Box::new(SpawnHandler::new(api, move |context, params| async move {
                handler(context, params)
            })),
        );
    }

    pub fn call_no_api<P, R>(
        &mut self,
        method: &str,
        handler: fn(context: Arc<ClientContext>, params: P) -> ApiResult<R>,
    ) where
        P: Send + DeserializeOwned + 'static,
        R: Send + Serialize + 'static,
    {
        self.sync_runners.insert(
            method.into(),
            Box::new(CallHandler::new(method_api(method), handler)),
        );

        #[cfg(feature = "node_interaction")]
        self.async_runners.insert(
            method.into(),
            Box::new(SpawnHandler::new(
                method_api(method),
                move |context, params| async move { handler(context, params) },
            )),
        );
    }

    pub fn call_no_args<R>(
        &mut self,
        method: &str,
        handler: fn(context: Arc<ClientContext>) -> ApiResult<R>,
    ) where
        R: TypeInfo + Send + Serialize + 'static,
    {
        let api = Method::from_types::<(), R>(method);
        self.sync_runners.insert(
            method.into(),
            Box::new(CallNoArgsHandler::new(api.clone(), handler)),
        );

        #[cfg(feature = "node_interaction")]
        self.async_runners.insert(
            method.into(),
            Box::new(SpawnNoArgsHandler::new(api, move |context| async move {
                handler(context)
            })),
        );
    }

    #[cfg(feature = "node_interaction")]
    pub fn spawn<P, R, F>(
        &mut self,
        method: &str,
        handler: fn(context: Arc<ClientContext>, params: P) -> F,
    ) where
        P: TypeInfo + Send + DeserializeOwned + 'static,
        R: TypeInfo + Send + Serialize + 'static,
        F: Send + Future<Output = ApiResult<R>> + 'static,
    {
        let api = Method::from_types::<P, R>(method);
        self.async_runners.insert(
            method.into(),
            Box::new(SpawnHandler::new(api.clone(), handler)),
        );

        self.sync_runners.insert(
            method.into(),
            Box::new(CallHandler::new(api, move |context, params| {
                context
                    .clone()
                    .async_runtime_handle
                    .block_on(handler(context, params))
            })),
        );
    }

    pub fn call_method<P, R>(
        &mut self,
        api: fn() -> api_doc::api::Method,
        handler: fn(context: std::sync::Arc<ClientContext>, params: P) -> ApiResult<R>,
    ) where
        P: TypeInfo + Send + DeserializeOwned + 'static,
        R: TypeInfo + Send + Serialize + 'static,
    {
        let api = api();
        self.sync_runners.insert(
            api.name.clone(),
            Box::new(CallHandler::new(api.clone(), handler)),
        );

        #[cfg(feature = "node_interaction")]
        self.async_runners.insert(
            api.name.clone(),
            Box::new(SpawnHandler::new(
                api,
                move |context, params| async move { handler(context, params) },
            )),
        );
    }

    pub fn spawn_method<P, R, F>(
        &mut self,
        api: fn() -> api_doc::api::Method,
        handler: fn(context: std::sync::Arc<ClientContext>, params: P) -> F,
    ) where
        P: TypeInfo + Send + DeserializeOwned + 'static,
        R: TypeInfo + Send + Serialize + 'static,
        F: Send + Future<Output = ApiResult<R>> + 'static,
    {
        let api = api();
        self.async_runners.insert(
            api.name.clone(),
            Box::new(SpawnHandler::new(api.clone(), handler)),
        );

        self.sync_runners.insert(
            api.name.clone(),
            Box::new(CallHandler::new(api, move |context, params| {
                context
                    .clone()
                    .async_runtime_handle
                    .block_on(handler(context, params))
            })),
        );
    }

    pub fn spawn_method_with_callback<P, R, F>(
        &mut self,
        api: fn() -> api_doc::api::Method,
        handler: fn(context: std::sync::Arc<ClientContext>, params: P, callback: Arc<Callback>) -> F,
    ) where
        P: TypeInfo + Send + DeserializeOwned + 'static,
        R: TypeInfo + Send + Serialize + 'static,
        F: Send + Future<Output = ApiResult<R>> + 'static,
    {
        let api = api();
        self.async_runners.insert(
            api.name.clone(),
            Box::new(SpawnHandlerCallback::new(api.clone(), handler)),
        );
    }

    #[cfg(feature = "node_interaction")]
    pub fn spawn_no_api<P, R, F>(
        &mut self,
        method: &str,
        handler: fn(context: Arc<ClientContext>, params: P) -> F,
    ) where
        P: Send + DeserializeOwned + 'static,
        R: Send + Serialize + 'static,
        F: Send + Future<Output = ApiResult<R>> + 'static,
    {
        self.async_runners.insert(
            method.into(),
            Box::new(SpawnHandler::new(method_api(method), handler)),
        );

        self.sync_runners.insert(
            method.into(),
            Box::new(CallHandler::new(
                method_api(method),
                move |context: Arc<ClientContext>, params: P| -> ApiResult<R> {
                    context
                        .clone()
                        .async_runtime_handle
                        .block_on(handler(context, params))
                },
            )),
        );
    }

    pub fn spawn_with_callback<P, R, F>(
        &mut self,
        method: &str,
        handler: fn(context: Arc<ClientContext>, params: P, callback: Arc<Callback>) -> F,
    ) where
        P: TypeInfo + Send + DeserializeOwned + 'static,
        R: TypeInfo + Send + Serialize + 'static,
        F: Send + Future<Output = ApiResult<R>> + 'static,
    {
        let api = Method::from_types::<P, R>(method);
        self.async_runners.insert(
            method.into(),
            Box::new(SpawnHandlerCallback::new(api, handler)),
        );
    }

    pub fn sync_dispatch(
        &self, context: Arc<ClientContext>, name: String, params_json: String
    ) -> JsonResponse {
        match self.sync_runners.get(&name) {
            Some(handler) => handler.handle(context, params_json.as_str()),
            None => JsonResponse::from_error(ApiError::unknown_function(&name)),
        }
    }

    #[cfg(feature = "node_interaction")]
    pub fn async_dispatch(
        &self,
        context: Arc<ClientContext>,
        function: String,
        params_json: String,
        request_id: u32,
        on_result: Box<ExternalCallback>,
    ) {
        match self.async_runners.get(&function) {
            Some(handler) => handler.handle(context, params_json, request_id, on_result),
            None => JsonResponse::from_error(ApiError::unknown_function(&function)).send(
                &*on_result,
                request_id,
            ),
        }
    }

    #[cfg(not(feature = "node_interaction"))]
    pub fn async_dispatch(
        &self,
        context: Arc<ClientContext>,
        function: String,
        params_json: String,
        request_id: u32,
        on_result: Box<ExternalCallback>,
    ) {
        self.sync_dispatch(context, function, params_json)
            .send(&*on_result, request_id, 1);
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
    dispatcher: &'a mut DispatchTable,
}

impl Registrar<'_> {
    pub fn t<T: ApiType>(&mut self) {
        self.module.types.push(T::api());
    }

    pub fn async_f<P, R, F>(
        &mut self,
        handler: fn(context: std::sync::Arc<ClientContext>, params: P) -> F,
        api: fn() -> api_info::Function,
    ) where
        P: ApiType + Send + DeserializeOwned + 'static,
        R: ApiType + Send + Serialize + 'static,
        F: Send + Future<Output = ApiResult<R>> + 'static,
    {
        self.t::<P>();
        self.t::<R>();
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

    pub fn f<P, R>(
        &mut self,
        handler: fn(context: std::sync::Arc<ClientContext>, params: P) -> ApiResult<R>,
        api: fn() -> api_info::Function,
    ) where
        P: ApiType + Send + DeserializeOwned + 'static,
        R: ApiType + Send + Serialize + 'static,
    {
        self.t::<P>();
        self.t::<R>();
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

    pub fn f_no_args<R>(
        &mut self,
        handler: fn(context: std::sync::Arc<ClientContext>) -> ApiResult<R>,
        api: fn() -> api_info::Function,
    ) where
        R: ApiType + Send + Serialize + 'static,
    {
        self.t::<R>();
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
