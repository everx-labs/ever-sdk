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

use crate::api::modules::register_modules;
use crate::api::request::Request;
use crate::client::{ClientConfig, ClientContext};
use crate::error::{ApiError, ApiResult};
use crate::{ContextHandle, ResponseHandler};
use api_info::{Module, API};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) trait SyncHandler {
    fn handle(&self, context: Arc<ClientContext>, params_json: &str) -> ApiResult<String>;
}

pub(crate) trait AsyncHandler {
    fn handle(
        &self,
        context: Arc<ClientContext>,
        params_json: String,
        request_id: u32,
        response_handler: ResponseHandler,
    );
}

// Handlers

pub(crate) struct RuntimeHandlers {
    sync_handlers: HashMap<String, Box<dyn SyncHandler + Sync>>,
    #[cfg(feature = "node_interaction")]
    async_handlers: HashMap<String, Box<dyn AsyncHandler + Sync>>,
    api: API,
}

impl RuntimeHandlers {
    fn new() -> RuntimeHandlers {
        let mut handlers = Self {
            sync_handlers: HashMap::new(),
            #[cfg(feature = "node_interaction")]
            async_handlers: HashMap::new(),
            api: API {
                version: "1.0.0".into(),
                modules: Vec::new(),
            },
        };
        register_modules(&mut handlers);
        handlers
    }

    pub fn add_module(&mut self, module: Module) {
        self.api.modules.push(module);
    }

    pub fn register_sync(&mut self, function_name: String, handler: Box<dyn SyncHandler + Sync>) {
        self.sync_handlers.insert(function_name, handler);
    }

    pub fn register_async(&mut self, function_name: String, handler: Box<dyn AsyncHandler + Sync>) {
        self.async_handlers.insert(function_name, handler);
    }
}

// Contexts
struct RuntimeContexts {
    next_context_handle: ContextHandle,
    contexts: HashMap<ContextHandle, Arc<ClientContext>>,
}

impl RuntimeContexts {
    fn new() -> Self {
        Self {
            next_context_handle: 1,
            contexts: HashMap::new(),
        }
    }
}

// Runtime

lazy_static! {
    static ref HANDLERS: RuntimeHandlers = RuntimeHandlers::new();
    static ref CONTEXTS: Mutex<RuntimeContexts> = Mutex::new(RuntimeContexts::new());
}

pub struct Runtime;

impl Runtime {
    fn handlers() -> &'static RuntimeHandlers {
        &HANDLERS
    }

    fn contexts() -> MutexGuard<'static, RuntimeContexts> {
        CONTEXTS.lock().unwrap()
    }

    pub fn dispatch_sync(
        context: Arc<ClientContext>,
        function_name: String,
        params_json: String,
    ) -> ApiResult<String> {
        match Self::handlers().sync_handlers.get(&function_name) {
            Some(handler) => handler.handle(context, params_json.as_str()),
            None => Err(ApiError::unknown_function(&function_name)),
        }
    }

    #[cfg(feature = "node_interaction")]
    pub fn dispatch_async(
        context: Arc<ClientContext>,
        function_name: String,
        params_json: String,
        request_id: u32,
        response_handler: ResponseHandler,
    ) {
        match Self::handlers().async_handlers.get(&function_name) {
            Some(handler) => handler.handle(context, params_json, request_id, response_handler),
            None => Request::new(response_handler, request_id)
                .finish_with_error(ApiError::unknown_function(&function_name)),
        }
    }

    #[cfg(not(feature = "node_interaction"))]
    pub fn dispatch_async(
        context: Arc<ClientContext>,
        function_name: String,
        params_json: String,
        request_id: u32,
        response_handler: ResponseHandler,
    ) {
        Request::new(response_handler, request_id).finish_with(Self::dispatch_sync(
            context,
            function_name,
            params_json,
        ));
    }

    pub fn api() -> &'static API {
        &Self::handlers().api
    }

    pub fn create_context(config_json: &str) -> ApiResult<ContextHandle> {
        let config = if !config_json.is_empty() {
            Some(
                serde_json::from_str::<ClientConfig>(config_json)
                    .map_err(|err| ApiError::invalid_params(config_json, err))?,
            )
        } else {
            None
        };
        let mut contexts = Self::contexts();
        let handle = contexts.next_context_handle;
        contexts.next_context_handle = handle.wrapping_add(1);
        let context = Arc::new(ClientContext::new(config)?);
        contexts.contexts.insert(handle, context.clone());
        Ok(handle)
    }

    pub fn destroy_context(handle: ContextHandle) {
        Self::contexts().contexts.remove(&handle);
    }

    pub fn required_context(context: ContextHandle) -> ApiResult<Arc<ClientContext>> {
        Ok(Arc::clone(
            Self::contexts()
                .contexts
                .get(&context)
                .ok_or(ApiError::invalid_context_handle(context))?,
        ))
    }
}
