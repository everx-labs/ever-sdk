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

pub use client::{ClientConfig, ClientContext};
pub use errors::{Error, ErrorCode};

pub(crate) use client_env::{FetchMethod, FetchResult, WebSocket};

use crate::error::ClientResult;
use crate::json_interface::runtime::Runtime;
use api_info::API;
use std::sync::Arc;

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ResultOfVersion {
    /// Core Library version
    pub version: String,
}

/// Returns Core Library version
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

/// Returns Core Library API reference
#[api_function]
pub fn get_api_reference(_context: Arc<ClientContext>) -> ClientResult<ResultOfGetApiReference> {
    Ok(ResultOfGetApiReference {
        api: Runtime::api().clone(),
    })
}

#[derive(ApiType, Serialize, Deserialize, Clone)]
pub struct BuildInfoDependency {
    /// Dependency name. Usually it is a crate name.
    pub name: String,
    /// Git commit hash of the related repository.
    pub git_commit: String,
}

#[derive(ApiType, Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct AppRequestParams<R: serde::Serialize> {
    pub app_request_id: u32,
    pub object_ref: String,
    pub request_data: R,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
#[serde(tag="type")]
pub enum RequestResult {
    Error { text: String },
    Ok { result: serde_json::Value }
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfResolveRequest {
    pub app_request_id: u32,
    pub result: RequestResult,
}

#[api_function]
pub async fn resolve_app_request(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfResolveRequest,
) -> ClientResult<()> {
    let request_id = params.app_request_id;
    let sender = context.app_requests
        .lock()
        .await
        .remove(&request_id)
        .ok_or(Error::no_such_request(request_id))?;

    sender.send(params.result)
        .map_err(|_| Error::can_not_send_request_result(request_id))
}
