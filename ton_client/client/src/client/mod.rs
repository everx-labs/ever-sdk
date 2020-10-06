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
    Client, ClientConfig, ClientContext, CryptoConfig, ExternalCallback, ResultOfCreateContext,
    ResultOfVersion, ResponseType,
    create_context,
};
pub use errors::{Error, ErrorCode};

pub(crate) use client_env::{ClientEnv, FetchMethod, FetchResult, WebSocket};

use crate::dispatch::{ModuleReg, Registrar};
use crate::error::ApiResult;
use serde_json::Value;
use std::sync::Arc;

/// BOC manipulation module.
#[derive(ApiType, Serialize)]
struct ResultOfGetApiReference {
    api: Value,
}

#[api_function]
fn get_api_reference(_context: Arc<ClientContext>) -> ApiResult<ResultOfGetApiReference> {
    Ok(ResultOfGetApiReference {
        api: serde_json::to_value(crate::client::api::get_api()).unwrap(),
    })
}

#[api_function]
fn version(_context: Arc<ClientContext>) -> ApiResult<ResultOfVersion> {
    Ok(ResultOfVersion {
        version: env!("CARGO_PKG_VERSION").to_owned(),
    })
}

/// BOC manipulation module.
#[derive(ApiModule)]
#[api_module(name = "client")]
pub(crate) struct ClientModule;

impl ModuleReg for ClientModule {
    fn reg(reg: &mut Registrar) {
        reg.f_no_args(get_api_reference, get_api_reference_api);
        reg.f_no_args(version, version_api);
        reg.f(unregister_callback, unregister_callback_api);
    }
}
