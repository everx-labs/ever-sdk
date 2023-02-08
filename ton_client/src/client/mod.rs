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
*/

mod client;
mod client_env;
pub(crate) mod errors;
#[cfg(not(feature = "wasm-base"))]
mod std_client_env;
pub(crate) mod storage;
#[cfg(not(feature = "wasm-base"))]
pub(crate) use std_client_env::{ClientEnv, LocalStorage};
#[cfg(feature = "wasm-base")]
mod wasm_client_env;
#[cfg(feature = "wasm-base")]
pub(crate) use wasm_client_env::{ClientEnv, LocalStorage};

#[cfg(not(feature = "wasm-base"))]
#[cfg(test)]
pub(crate) use crate::client::network_mock::NetworkMock;

#[cfg(test)]
mod tests;

#[cfg(not(feature = "wasm-base"))]
#[cfg(test)]
mod network_mock;

pub use client::{ClientConfig, ClientContext};
pub use errors::{Error, ErrorCode};

use crate::error::ClientResult;
use crate::json_interface::runtime::Runtime;
use api_info::API;
pub(crate) use client::AppObject;
pub(crate) use client_env::{FetchMethod, FetchResult, WebSocket};
use std::sync::Arc;

pub(crate) const LOCAL_STORAGE_DEFAULT_DIR_NAME: &str = ".tonclient";

lazy_static! {
    static ref BINDING_CONFIG: std::sync::RwLock<Option<BindingConfig>> =
        std::sync::RwLock::new(None);
}

pub fn core_version() -> String {
    env!("CARGO_PKG_VERSION").into()
}

pub fn binding_config() -> Option<BindingConfig> {
    (*BINDING_CONFIG).read().unwrap().clone()
}

fn binding_config_is_some() -> bool {
    (*BINDING_CONFIG).read().unwrap().is_some()
}

pub fn update_binding_config(config: &BindingConfig) {
    if !binding_config_is_some() {
        *(*BINDING_CONFIG).write().unwrap() = Some(config.clone());
    }
}

#[derive(Clone, Default, Deserialize, Serialize, Debug, ApiType)]
pub struct BindingConfig {
    #[serde(default)]
    pub library: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ResultOfVersion {
    /// Core Library version
    pub version: String,
}

/// Returns Core Library version
#[api_function]
pub fn version(_context: Arc<ClientContext>) -> ClientResult<ResultOfVersion> {
    Ok(ResultOfVersion {
        version: core_version(),
    })
}

#[derive(ApiType, Default, Serialize, Deserialize)]
pub struct ResultOfGetApiReference {
    pub api: API,
}

/// Returns Core Library API reference
#[api_function]
pub fn get_api_reference(_context: Arc<ClientContext>) -> ClientResult<ResultOfGetApiReference> {
    Ok(ResultOfGetApiReference {
        api: Runtime::api().clone(),
    })
}

#[derive(ApiType, Default, Serialize, Deserialize, Clone)]
pub struct BuildInfoDependency {
    /// Dependency name. Usually it is a crate name.
    pub name: String,
    /// Git commit hash of the related repository.
    pub git_commit: String,
}

#[derive(ApiType, Default, Serialize, Deserialize, Clone)]
pub struct ResultOfBuildInfo {
    /// Build number assigned to this build by the CI.
    build_number: u32,
    /// Fingerprint of the most important dependencies.
    dependencies: Vec<BuildInfoDependency>,
}

/// Returns detailed information about this build.
#[api_function]
pub fn build_info(_context: Arc<ClientContext>) -> ClientResult<ResultOfBuildInfo> {
    Ok(
        serde_json::from_str(include_build_info!()).unwrap_or(ResultOfBuildInfo {
            build_number: 0,
            dependencies: vec![],
        }),
    )
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfAppRequest {
    /// Request ID. Should be used in `resolve_app_request` call
    pub app_request_id: u32,
    /// Request describing data
    pub request_data: serde_json::Value,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
#[serde(tag = "type")]
pub enum AppRequestResult {
    /// Error occurred during request processing
    Error {
        /// Error description
        text: String,
    },
    /// Request processed successfully
    Ok {
        /// Request processing result
        result: serde_json::Value,
    },
}

impl Default for AppRequestResult {
    fn default() -> Self {
        AppRequestResult::Error {
            text: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfResolveAppRequest {
    /// Request ID received from SDK
    pub app_request_id: u32,
    /// Result of request processing
    pub result: AppRequestResult,
}

/// Resolves application request processing result
#[api_function]
pub async fn resolve_app_request(
    context: Arc<ClientContext>,
    params: ParamsOfResolveAppRequest,
) -> ClientResult<()> {
    let request_id = params.app_request_id;
    let sender = context
        .app_requests
        .lock()
        .await
        .remove(&request_id)
        .ok_or(Error::no_such_request(request_id))?;

    sender
        .send(params.result)
        .map_err(|_| Error::can_not_send_request_result(request_id))
}

/// Returns Core Library API reference
#[api_function]
pub fn config(context: Arc<ClientContext>) -> ClientResult<ClientConfig> {
    Ok(context.config.clone())
}
