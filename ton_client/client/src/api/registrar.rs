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

use crate::api::handlers::{
    CallHandler, CallNoArgsHandler, SpawnHandler, SpawnHandlerCallback, SpawnNoArgsHandler,
};
use crate::api::request::Request;
use crate::api::runtime::{RuntimeHandlers};
use crate::client::ClientContext;
use crate::error::ApiResult;
use api_info::{ApiModule, ApiType, Module};
use serde::de::DeserializeOwned;
use serde::Serialize;
#[cfg(feature = "node_interaction")]
use std::future::Future;
use std::sync::Arc;

pub(crate) struct ModuleReg<'h> {
    handlers: &'h mut RuntimeHandlers,
    module: Module,
}

impl<'h> ModuleReg<'h> {
    pub fn new<M: ApiModule>(handlers: &'h mut RuntimeHandlers) -> Self {
        Self {
            module: M::api(),
            handlers,
        }
    }

    pub fn register(self) {
        self.handlers.add_module(self.module);
    }

    pub fn register_type<T: ApiType>(&mut self) {
        self.module.types.push(T::api());
    }

    pub fn register_async_fn<P, R, F>(
        &mut self,
        handler: fn(context: std::sync::Arc<ClientContext>, params: P) -> F,
        api: fn() -> api_info::Function,
    ) where
        P: ApiType + Send + DeserializeOwned + 'static,
        R: ApiType + Send + Serialize + 'static,
        F: Send + Future<Output = ApiResult<R>> + 'static,
    {
        self.register_type::<P>();
        self.register_type::<R>();
        let function = api();
        let name = format!("{}.{}", self.module.name, function.name);
        self.module.functions.push(function);

        self.handlers.register_async(name.clone(), Box::new(SpawnHandler::new(handler)));
        self.handlers.register_sync(
            name,
            Box::new(CallHandler::new(move |context, params| {
                context
                    .clone()
                    .async_runtime_handle
                    .block_on(handler(context, params))
            })),
        );
    }

    pub fn register_async_fn_with_callback<P, R, F>(
        &mut self,
        handler: fn(context: std::sync::Arc<ClientContext>, params: P, callback: Arc<Request>) -> F,
        api: fn() -> api_info::Function,
    ) where
        P: ApiType + Send + DeserializeOwned + 'static,
        R: ApiType + Send + Serialize + 'static,
        F: Send + Future<Output = ApiResult<R>> + 'static,
    {
        self.register_type::<P>();
        self.register_type::<R>();
        let function = api();
        let name = format!("{}.{}", self.module.name, function.name);
        self.module.functions.push(function);
        self.handlers.register_async(name.clone(), Box::new(SpawnHandlerCallback::new(handler)));
    }

    pub fn register_sync_fn<P, R>(
        &mut self,
        handler: fn(context: std::sync::Arc<ClientContext>, params: P) -> ApiResult<R>,
        api: fn() -> api_info::Function,
    ) where
        P: ApiType + Send + DeserializeOwned + 'static,
        R: ApiType + Send + Serialize + 'static,
    {
        self.register_type::<P>();
        self.register_type::<R>();
        let function = api();
        let name = format!("{}.{}", self.module.name, function.name);
        self.module.functions.push(function);

        self.handlers.register_sync(name.clone(), Box::new(CallHandler::new(handler)));

        #[cfg(feature = "node_interaction")]
        self.handlers.register_async(
            name.clone(),
            Box::new(SpawnHandler::new(move |context, params| async move {
                handler(context, params)
            })),
        );
    }

    pub fn register_sync_fn_without_args<R>(
        &mut self,
        handler: fn(context: std::sync::Arc<ClientContext>) -> ApiResult<R>,
        api: fn() -> api_info::Function,
    ) where
        R: ApiType + Send + Serialize + 'static,
    {
        self.register_type::<R>();
        let function = api();
        let name = format!("{}.{}", self.module.name, function.name);
        self.module.functions.push(function);

        self.handlers.register_sync(name.clone(), Box::new(CallNoArgsHandler::new(handler)));

        #[cfg(feature = "node_interaction")]
        self.handlers.register_async(
            name.clone(),
            Box::new(SpawnNoArgsHandler::new(move |context| async move {
                handler(context)
            })),
        );
    }
}
