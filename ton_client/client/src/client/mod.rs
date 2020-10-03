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

mod api;
mod client;
mod client_env;
mod errors;
mod std_client_env;
mod tests;

pub use api::get_api;
pub use client::{
    create_context, Callback, Client, ClientConfig, ClientContext, CryptoConfig,
    ParamsOfUnregisterCallback, ResultOfCreateContext, ResultOfVersion,
};
pub use errors::{Error, ErrorCode};

pub(crate) use client_env::{ClientEnv, FetchMethod, FetchResult, WebSocket};

use crate::dispatch::DispatchTable;
use crate::error::ApiResult;
use serde_json::Value;
use std::sync::Arc;

pub fn register_callback(
    context: std::sync::Arc<ClientContext>,
    _params_json: String,
    request_id: u32,
    on_result: Box<Callback>,
) {
    context.callbacks.insert(request_id, on_result.into());
}

#[function_info]
pub fn unregister_callback(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfUnregisterCallback,
) -> ApiResult<()> {
    context.callbacks.remove(&params.callback_id);
    Ok(())
}

/// BOC manipulation module.
#[derive(TypeInfo, Serialize)]
struct ResultOfGetApiReference {
    api: Value,
}

#[function_info]
fn get_api_reference(_context: Arc<ClientContext>) -> ApiResult<ResultOfGetApiReference> {
    Ok(ResultOfGetApiReference {
        api: serde_json::to_value(crate::client::api::get_api()).unwrap(),
    })
}

#[function_info]
fn version(_context: Arc<ClientContext>) -> ApiResult<ResultOfVersion> {
    Ok(ResultOfVersion {
        version: env!("CARGO_PKG_VERSION").to_owned(),
    })
}

/// BOC manipulation module.
#[derive(TypeInfo)]
#[type_info(name = "client")]
struct ClientModule;

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.register_module::<ClientModule>(|reg| {
        reg.f_no_args(get_api_reference, get_api_reference_info);
        reg.f_no_args(version, version_info);
        reg.f(unregister_callback, unregister_callback_info);
    });
    handlers.call_raw_async("client.register_callback", register_callback);
}
