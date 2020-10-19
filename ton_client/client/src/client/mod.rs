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

mod client;
mod client_env;
pub(crate) mod errors;
#[cfg(not(feature = "wasm"))]
mod std_client_env;
#[cfg(not(feature = "wasm"))]
pub(crate) use std_client_env::ClientEnv;
#[cfg(feature = "wasm")]
mod wasm_client_env;
#[cfg(feature = "wasm")]
pub(crate) use wasm_client_env::ClientEnv;

#[cfg(test)]
mod tests;

pub use client::{
    ClientConfig, ClientContext, CryptoConfig,
};
pub use errors::{Error, ErrorCode};

pub(crate) use client_env::{FetchMethod, FetchResult, WebSocket};

use crate::error::ClientResult;
use std::sync::Arc;
use api_info::API;
use crate::c_interface::api_reference::ApiReducer;
use crate::c_interface::runtime::Runtime;

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ResultOfVersion {
    /// core version
    pub version: String,
}

#[api_function]
pub fn version(_context: Arc<ClientContext>) -> ClientResult<ResultOfVersion> {
    Ok(ResultOfVersion {
        version: env!("CARGO_PKG_VERSION").to_owned(),
    })
}

#[derive(ApiType, Serialize, Deserialize)]
pub struct ResultOfGetApiReference {
    pub api: API,
}

#[api_function]
pub fn get_api_reference(_context: Arc<ClientContext>) -> ClientResult<ResultOfGetApiReference> {
    let api = ApiReducer::build(Runtime::api());
    Ok(ResultOfGetApiReference {
        api,
    })
}

