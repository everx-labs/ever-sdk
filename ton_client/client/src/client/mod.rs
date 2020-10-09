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
mod std_client_env;
#[cfg(test)]
mod tests;

pub use client::{ClientConfig, ClientContext, CryptoConfig};
pub use errors::{Error, ErrorCode};

pub(crate) use client_env::{ClientEnv, FetchMethod, FetchResult, WebSocket};

use crate::error::ClientResult;
use crate::json_interface::runtime::Runtime;
use api_info::API;
use std::sync::Arc;

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
    Ok(ResultOfGetApiReference {
        api: Runtime::api().clone(),
    })
}

#[derive(ApiType, Serialize, Deserialize, Clone)]
pub struct ResultOfBuildInfo {
    pub build_info: serde_json::Value,
}

#[api_function]
pub fn build_info(_context: Arc<ClientContext>) -> ClientResult<ResultOfBuildInfo> {
    Ok(ResultOfBuildInfo {
        build_info: serde_json::from_str(include_str!("../build_info.json")).unwrap_or(json!({}))
    })
}

