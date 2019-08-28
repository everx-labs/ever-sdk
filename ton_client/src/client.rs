use dispatch::DispatchTable;
use ::{JsonResponse, InteropContext};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use types::{ApiResult, ApiError};

fn create_handlers() -> DispatchTable {
    let mut handlers = DispatchTable::new();
    crate::setup::register(&mut handlers);
    crate::crypto::register(&mut handlers);
    crate::contracts::register(&mut handlers);
    crate::queries::register(&mut handlers);
    handlers
}

lazy_static! {
    static ref HANDLERS: DispatchTable = create_handlers();
}


fn sync_request(context: &mut Context, method: String, params_json: String) -> JsonResponse {
    HANDLERS.sync_dispatch(context, method, params_json)
}

pub(crate) struct Context {
    pub handle: u32
}

pub(crate) struct Client {
    next_context_handle: InteropContext,
    contexts: HashMap<InteropContext, Context>,
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
        self.contexts.insert(handle, Context {
            handle
        });
        handle
    }

    pub fn destroy_context(&mut self, handle: InteropContext) {
        self.required_context(handle);
        self.contexts.remove(&handle);
    }

    pub fn required_context(&mut self, context: InteropContext) -> ApiResult<&mut Context> {
        self.contexts.get_mut(&context).ok_or(
            ApiError::invalid_context_handle(context)
        )
    }

    pub fn json_request(&mut self, context: InteropContext, method_name: String, params_json: String) -> JsonResponse {
        let context = self.required_context(context);
        match context {
            Ok(context) => sync_request(context, method_name, params_json),
            Err(err) => JsonResponse::from_error(err)
        }
    }

}


