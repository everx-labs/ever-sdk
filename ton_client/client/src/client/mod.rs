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
mod errors;
mod std_client_env;
mod client_env;

pub use client::{
    Client, ClientConfig, ClientContext, CryptoConfig, ResultOfCreateContext,
    ResultOfVersion, ExternalCallback, ParamsOfUnregisterCallback, ResponseType,
    create_context,
};
pub use errors::{ErrorCode, Error};

pub(crate) use client_env::{ClientEnv, FetchMethod, FetchResult, WebSocket};

use crate::dispatch::DispatchTable;

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.call_no_args("client.get_api_reference", |_context| Ok(crate::get_api()));
    handlers.call_no_args("client.version", |_| {
        Ok(ResultOfVersion {
            version: env!("CARGO_PKG_VERSION").to_owned(),
        })
    });
}
