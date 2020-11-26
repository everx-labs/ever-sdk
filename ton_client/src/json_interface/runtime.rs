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

use super::modules::register_modules;
use super::request::Request;
use crate::client::{ClientConfig, ClientContext, Error};
use crate::error::ClientResult;
use crate::ContextHandle;
use api_info::{Module, API};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) trait SyncHandler {
    fn handle(&self, context: Arc<ClientContext>, params_json: &str) -> ClientResult<String>;
}

pub(crate) trait AsyncHandler {
    fn handle(&self, context: Arc<ClientContext>, params_json: String, request: Request);
}

// Handlers

pub(crate) struct RuntimeHandlers {
    sync_handlers: HashMap<String, Box<dyn SyncHandler + Sync>>,
    async_handlers: HashMap<String, Box<dyn AsyncHandler + Sync>>,
    api: API,
}

impl RuntimeHandlers {
    fn new() -> RuntimeHandlers {
        let mut handlers = Self {
            sync_handlers: HashMap::new(),
            async_handlers: HashMap::new(),
            api: API {
                version: env!("CARGO_PKG_VERSION").to_owned(),
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
    ) -> ClientResult<String> {
        match Self::handlers().sync_handlers.get(&function_name) {
            Some(handler) => handler.handle(context, params_json.as_str()),
            None => Err(Error::unknown_function(&function_name)),
        }
    }

    pub fn dispatch_async(
        context: Arc<ClientContext>,
        function_name: String,
        params_json: String,
        request: Request,
    ) {
        match Self::handlers().async_handlers.get(&function_name) {
            Some(handler) => handler.handle(context, params_json, request),
            None => request.finish_with_error(Error::unknown_function(&function_name)),
        }
    }

    pub fn api() -> &'static API {
        &Self::handlers().api
    }

    pub fn create_context(config_json: &str) -> ClientResult<ContextHandle> {
        let config_json = if !config_json.is_empty() {
            config_json
        } else {
            "{}"
        };
        let config = serde_json::from_str::<ClientConfig>(config_json)
            .map_err(|err| Error::invalid_params(config_json, err))?;

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

    pub fn required_context(context: ContextHandle) -> ClientResult<Arc<ClientContext>> {
        Ok(Arc::clone(
            Self::contexts()
                .contexts
                .get(&context)
                .ok_or(Error::invalid_context_handle(context))?,
        ))
    }
}
