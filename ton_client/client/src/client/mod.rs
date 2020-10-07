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
mod errors;
mod std_client_env;
#[cfg(test)]
mod tests;

pub use client::{
    Client, ClientConfig, ClientContext, ContextHandle, CryptoConfig, Request, ResponseHandler,
    ResponseType, ResultOfVersion, StringData,
};
pub use errors::{Error, ErrorCode};

pub(crate) use client::parse_params;
pub(crate) use client_env::{ClientEnv, FetchMethod, FetchResult, WebSocket};

use crate::error::ApiResult;
use std::sync::Arc;

#[api_function]
pub fn version(_context: Arc<ClientContext>) -> ApiResult<ResultOfVersion> {
    Ok(ResultOfVersion {
        version: env!("CARGO_PKG_VERSION").to_owned(),
    })
}
