/*
 * Copyright 2018-2021 TON Labs LTD.
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
use crate::error::{ClientError, ClientResult};
use crate::{ContextHandle, ResponseType};
use api_info::{Module, API};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
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

// Used as a `request_ptr` in case when async handler invoker via `dispatch_sync`
struct AsyncAsSyncHandler {
    // Used to send async result to the waiting sync handler.
    result_sender: Sender<ClientResult<String>>,
    // Indicates that async result (success or error) is already sent to the sync handler
    // and async handler should ignore all following responses.
    result_sent: bool,
}

impl AsyncAsSyncHandler {
    fn dispatch(
        context: Arc<ClientContext>,
        function_name: String,
        params_json: String,
    ) -> ClientResult<String> {
        let (result_sender, result_receiver) = std::sync::mpsc::channel::<ClientResult<String>>();
        // Create state, unbox it into raw pointer and send to the `dispatch_sync`
        let handler = Box::into_raw(Box::new(Self {
            result_sender,
            result_sent: false,
        }));
        Runtime::dispatch_async(
            context,
            function_name,
            params_json,
            Request::new_with_ptr(handler as *const (), Self::response_handler),
        );
        let result = result_receiver
            .recv()
            .unwrap_or_else(|err| Err(Error::can_not_receive_request_result(err)));
        result
    }

    fn handle_response(&mut self, params_json: String, response_type: u32) {
        if self.result_sent {
            return;
        }
        let result = if response_type == ResponseType::Success as u32 {
            Ok(params_json)
        } else if response_type == ResponseType::Error as u32 {
            Err(serde_json::from_str::<ClientError>(&params_json)
                .unwrap_or_else(|err| Error::callback_params_cant_be_converted_to_json(err)))
        } else {
            return;
        };
        let _ = self.result_sender.send(result);
        self.result_sent = true;
    }

    fn response_handler(
        request_ptr: *const (),
        params_json: String,
        response_type: u32,
        finished: bool,
    ) {
        let handler = request_ptr as *mut Self;
        if let Some(handler) = unsafe { handler.as_mut() } {
            handler.handle_response(params_json, response_type);
        }
        if finished {
            // Box handler from raw pointer and drop it
            let _ = unsafe { Box::from_raw(handler) };
        }
    }
}

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
            None => AsyncAsSyncHandler::dispatch(context, function_name, params_json),
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
