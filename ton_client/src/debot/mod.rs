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
mod browser;
mod context;
mod debot_abi;
mod dengine;
mod errors;
mod routines;
#[cfg(test)]
mod tests;

pub use dengine::DEngine;
pub use context::{DContext, STATE_EXIT, STATE_ZERO};
pub use action::DAction;
pub use browser::BrowserCallbacks;

use crate::{ClientConfig, ClientContext};
use crate::error::ClientResult;
use std::sync::Arc;
use std::collections::HashMap;

pub type DebotResult<T> = Result<T, String>;
pub type DebotHandle = u32;

pub struct DebotConfig {
    handles: HashMap<DebotHandle, DEngine>,
    next_handle: DebotHandle,
}

impl DebotConfig {
    pub fn new() -> Self {
        DebotConfig {
            handles: HashMap::new(),
            next_handle: 1,
        }
    }
}

pub struct ParamsOfDebotStart {
    url: String,
    abi: Option<String>,
    address: String,
    callbacks: Box<dyn BrowserCallbacks>,
}

pub struct ResultOfDebotStart {
    pub debot_handle: DebotHandle,
}

pub struct ParamsOfDebotExecute {
    pub debot_handle: DebotHandle,
    pub action: DAction,
}

pub struct ResultOfDebotExecute {

}

pub struct ParamsOfDebotFetch {

}

pub struct ResultOfDebotFetch {

}

pub async fn debot_start(
    context: Arc<ClientContext>,
    params: ParamsOfDebotStart,
) -> DebotResult<ResultOfDebotStart> {
    let callbacks = params.callbacks;
    let mut dengine = DEngine::new(params.address, params.abi, &params.url, callbacks);
    dengine.start().await?;
    
    let debots = context.debot_config.write().unwrap();
    debots.handles[debots.next_handle] = dengine;
    let debot_handle = debots.next_handle;
    debots.next_handle += 1;

    Ok(ResultOfDebotStart { debot_handle })
}

pub async fn debot_execute_action(
    context: Arc<ClientContext>,
    params: ParamsOfDebotExecute,
) -> DebotResult<()> {
    let debots = context.debot_config.write().unwrap();
    let dengine = debots.handles.get_mut(params.debot_handle);
    dengine.execute_action(params.action)
}

/*
pub async fn debot_fetch(
    context: Arc<ClientContext>,
    params: ParamsOfDebotFetch,
) -> DebotResult<ResultOfDebotFetch> {

}
*/
