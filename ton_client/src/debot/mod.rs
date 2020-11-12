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
mod action;
mod adapter;
mod browser;
mod context;
mod debot_abi;
mod dengine;
mod drequest;
mod dresponse;
mod errors;
mod routines;
#[cfg(test)]
mod tests;

pub use action::DAction;
pub use browser::BrowserCallbacks;
pub use context::{DContext, STATE_EXIT, STATE_ZERO};
pub use dengine::DEngine;
pub use drequest::DebotBrowserRequest;
pub use dresponse::DebotBrowserResponse;

use crate::error::ClientResult;
use crate::json_interface::request::Request;
use crate::ClientContext;
use adapter::DebotBrowserAdapter;
use errors::{error, ErrorCode};
use std::collections::HashMap;
use std::sync::Arc;

pub type DebotHandle = u32;

pub struct DebotContext {
    handles: HashMap<DebotHandle, DEngine>,
    next_handle: DebotHandle,
}

impl DebotContext {
    pub fn new() -> Self {
        DebotContext {
            handles: HashMap::new(),
            next_handle: 1,
        }
    }
}

#[derive(Serialize, Deserialize, Default, ApiType)]
pub struct ParamsOfDebotStart {
    url: String,
    abi: Option<String>,
    address: String,
    app_ref: String,
}

#[derive(Serialize, Deserialize, Default, ApiType)]
pub struct ResultOfDebotStart {
    pub debot_handle: DebotHandle,
}

#[derive(Serialize, Deserialize, Default, ApiType)]
pub struct ParamsOfDebotExecute {
    pub debot_handle: DebotHandle,
    pub action: DAction,
}

#[derive(Serialize, Deserialize, Default, ApiType)]
pub struct ResultOfDebotExecute {}

/*
pub struct ParamsOfDebotFetch {

}

pub struct ResultOfDebotFetch {

}
*/

#[api_function]
pub async fn debot_start(
    context: Arc<ClientContext>,
    params: ParamsOfDebotStart,
    callback: Arc<Request>,
) -> ClientResult<ResultOfDebotStart> {
    let context_copy = context.clone();
    let callback = move |request| {
        let callback = callback.clone();
        let context = context_copy.clone();
        async move { context.app_request(&callback, request).await }
    };

    let browser_callbacks = Box::new(DebotBrowserAdapter::new(callback, params.app_ref));
    let mut dengine = DEngine::new(params.address, params.abi, &params.url, browser_callbacks);
    dengine
        .start()
        .await
        .map_err(|e| error(ErrorCode::DebotStartFailed, e))?;

    let mut ctx = context.debot_ctx.write().await;
    ctx.handles.insert(ctx.next_handle, dengine);
    let debot_handle = ctx.next_handle;
    ctx.next_handle += 1;

    Ok(ResultOfDebotStart { debot_handle })
}

#[api_function]
pub async fn debot_execute_action(
    context: Arc<ClientContext>,
    params: ParamsOfDebotExecute,
) -> ClientResult<()> {
    let mut ctx = context.debot_ctx.write().await;
    let dengine = ctx.handles.get_mut(&params.debot_handle).ok_or(error(
        ErrorCode::DebotInvalidHandle,
        "debot handle is invalid".to_string(),
    ))?;
    dengine
        .execute_action(&params.action)
        .await
        .map_err(|e| error(ErrorCode::DebotExecutionFailed, e))
}

/*
pub async fn debot_fetch(
    context: Arc<ClientContext>,
    params: ParamsOfDebotFetch,
) -> DebotResult<ResultOfDebotFetch> {

}
*/
