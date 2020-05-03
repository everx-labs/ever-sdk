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

use crate::dispatch::DispatchTable;
use crate::types::{ApiResult, ApiError};
use super::{JsonResponse, InteropContext};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use ton_sdk::NodeClient;

#[cfg(feature = "node_interaction")]
use tokio::runtime::Runtime;

fn create_handlers() -> DispatchTable {
    let mut handlers = DispatchTable::new();
    crate::setup::register(&mut handlers);
    crate::crypto::register(&mut handlers);
    crate::contracts::register(&mut handlers);
    crate::tvm::register(&mut handlers);
    crate::cell::register(&mut handlers);

    #[cfg(feature = "node_interaction")]
    crate::queries::register(&mut handlers);

    handlers
}

lazy_static! {
    static ref HANDLERS: DispatchTable = create_handlers();
}


fn sync_request(context: &mut ClientContext, method: String, params_json: String) -> JsonResponse {
    HANDLERS.sync_dispatch(context, method, params_json)
}

pub(crate) struct ClientContext {
    pub client: Option<NodeClient>,
    #[cfg(feature = "node_interaction")]
    pub runtime: Option<Runtime>,
    pub handle: u32
}

impl ClientContext {
    pub fn get_client(&self) -> ApiResult<&NodeClient> {
        self.client.as_ref().ok_or(ApiError::sdk_not_init())
    }

    #[cfg(feature = "node_interaction")]
    pub fn take_runtime(&mut self) -> ApiResult<Runtime> {
        self.runtime.take().ok_or(ApiError::sdk_not_init())
    }
}

pub(crate) struct Client {
    next_context_handle: InteropContext,
    contexts: HashMap<InteropContext, ClientContext>,
}


lazy_static! {
    static ref CLIENT: Mutex<Client> = Mutex::new(Client::new());
}

impl Client {
    fn new() -> Self {
        Self {
            next_context_handle: 1,
            contexts: HashMap::new(),
        }
    }

    pub fn shared() -> MutexGuard<'static, Client> {
        CLIENT.lock().unwrap()
    }

    // Contexts

    pub fn create_context(&mut self) -> InteropContext {
        let handle = self.next_context_handle;
        self.next_context_handle = handle.wrapping_add(1);

        #[cfg(feature = "node_interaction")]
        self.contexts.insert(handle, ClientContext {
            handle,
            client: None,
            runtime: None,
        });

        #[cfg(not(feature = "node_interaction"))]
        self.contexts.insert(handle, ClientContext {
            handle,
            client: None,
        });

        handle
    }

    pub fn destroy_context(&mut self, handle: InteropContext) {
        self.required_context(handle).unwrap();
        if self.contexts.len() == 1 {
            self.json_sync_request(handle, "uninit".to_owned(), "{}".to_owned());
        }
        self.contexts.remove(&handle);
    }

    pub fn required_context(&mut self, context: InteropContext) -> ApiResult<&mut ClientContext> {
        self.contexts.get_mut(&context).ok_or(
            ApiError::invalid_context_handle(context)
        )
    }

    pub fn json_sync_request(&mut self, context: InteropContext, method_name: String, params_json: String) -> JsonResponse {
        let context = self.required_context(context);
        match context {
            Ok(context) => sync_request(context, method_name, params_json),
            Err(err) => JsonResponse::from_error(err)
        }
    }

}


