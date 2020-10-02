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

pub use client::{
    create_context, Callback, Client, ClientConfig, ClientContext, CryptoConfig,
    ParamsOfUnregisterCallback, ResultOfCreateContext, ResultOfVersion,
};
pub use errors::{Error, ErrorCode};

pub(crate) use client_env::{ClientEnv, FetchMethod, FetchResult, WebSocket};

use crate::dispatch::DispatchTable;
use crate::error::ApiResult;

pub fn register_callback(
    context: std::sync::Arc<ClientContext>,
    _params_json: String,
    request_id: u32,
    on_result: Box<Callback>,
) {
    context.callbacks.insert(request_id, on_result.into());
}

pub fn unregister_callback(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfUnregisterCallback,
) -> ApiResult<()> {
    context.callbacks.remove(&params.callback_id);
    Ok(())
}

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.call_no_args("client.get_api_reference", |_context| {
        Ok(crate::client::api::get_api())
    });
    handlers.call_no_args("client.version", |_| {
        Ok(ResultOfVersion {
            version: env!("CARGO_PKG_VERSION").to_owned(),
        })
    });

    handlers.call_raw_async("client.register_callback", register_callback);

    handlers.call("client.unregister_callback", unregister_callback);
}
