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
mod action;
mod adapter;
mod browser;
mod context;
mod debot_abi;
mod dengine;
mod errors;
mod routines;
#[cfg(test)]
mod tests;

pub use action::DAction;
pub use browser::BrowserCallbacks;
pub use context::{DContext, STATE_EXIT, STATE_ZERO};
pub use dengine::DEngine;

use crate::error::ClientResult;
use crate::ClientContext;
use crate::client::AppObject;
use crate::crypto::KeyPair;
use adapter::DebotBrowserAdapter;
use errors::{error, ErrorCode};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Default, ApiType)]
pub struct DebotHandle(u32);

#[derive(Serialize, Deserialize, Default, ApiType)]
pub struct ParamsOfDebotStart {
    url: String,
    abi: Option<String>,
    address: String,
    app_ref: String,
}

#[derive(Serialize, Deserialize, Default, ApiType)]
pub struct ResultOfDebotStart {
    pub debot_handle: DebotHandle,
}

#[derive(Serialize, Deserialize, Default, ApiType)]
pub struct ParamsOfDebotExecute {
    pub debot_handle: DebotHandle,
    pub action: DAction,
}

/*
pub struct ParamsOfDebotFetch {

}

pub struct ResultOfDebotFetch {

}
*/

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub enum ResultOfAppDebotBrowser {
    Input { value: String },
    LoadKey { keys: KeyPair },
    InvokeDebot,
}

#[derive(Serialize, Deserialize, Clone, ApiType)]
pub enum ParamsOfAppDebotBrowser {
    Log {
        msg: String,
    },
    Switch {
        ctx_id: u8,
    },
    ShowAction {
        action: DAction,
    },
    Input {
        prefix: String,
    },
    LoadKey,
    InvokeDebot {
        debot_addr: String,
        action: DAction,
    },
}

#[api_function]
pub(crate) async fn debot_start(
    context: Arc<ClientContext>,
    params: ParamsOfDebotStart,
    app_object: AppObject<ParamsOfAppDebotBrowser, ResultOfAppDebotBrowser>,
) -> ClientResult<ResultOfDebotStart> {
    let browser_callbacks = Box::new(DebotBrowserAdapter::new(app_object));
    let mut dengine = DEngine::new(params.address, params.abi, &params.url, browser_callbacks);
    dengine
        .start()
        .await
        .map_err(|e| error(ErrorCode::DebotStartFailed, e))?;

    let handle = context.get_next_id();
    context.debots.insert(handle, Mutex::new(dengine));

    Ok(ResultOfDebotStart { debot_handle: DebotHandle(handle) })
}

#[api_function]
pub async fn debot_execute_action(
    context: Arc<ClientContext>,
    params: ParamsOfDebotExecute,
) -> ClientResult<()> {
    let mutex = context.debots.get(&params.debot_handle.0).ok_or(error(
        ErrorCode::DebotInvalidHandle,
        "debot handle is invalid".to_string(),
    ))?;
    let mut dengine = mutex
        .1
        .lock()
        .await;
    
    dengine
        .execute_action(&params.action)
        .await
        .map_err(|e| error(ErrorCode::DebotExecutionFailed, e))
}

/*
pub async fn debot_fetch(
    context: Arc<ClientContext>,
    params: ParamsOfDebotFetch,
) -> DebotResult<ResultOfDebotFetch> {

}
*/
