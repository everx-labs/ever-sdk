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

pub use ton_sdk::NetworkConfig;
pub use client::{
    Client, ClientConfig, ClientContext, CryptoConfig, ResultOfCreateContext,
    ResultOfVersion, Callback, ParamsOfUnregisterCallback,
    create_context, register_callback, unregister_callback
};
pub use errors::{ErrorCode, Error};

pub(crate) use client_env::{ClientEnv, FetchMethod, FetchResult, WebSocket};
