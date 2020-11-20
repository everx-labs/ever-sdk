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

pub use action::DAction;
pub use browser::BrowserCallbacks;
pub use context::{DContext, STATE_EXIT, STATE_ZERO};
pub use dengine::DEngine;
pub use errors::Error;

use crate::error::ClientResult;
use crate::ClientContext;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Default, ApiType, Clone)]
pub struct DebotHandle(u32);

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct DebotAction {
    pub description: String,
    pub name: String,
    pub action_type: u8,
    pub to: u8,
    pub attributes: String,
    pub misc: String,
}

impl From<DAction> for DebotAction {
    fn from(daction: DAction) -> Self {
        Self {
            description: daction.desc,
            name: daction.name,
            action_type: daction.action_type as u8,
            to: daction.to,
            attributes: daction.attrs,
            misc: daction.misc,
        }
    }
}

impl Into<DAction> for DebotAction {
    fn into(self) -> DAction {
        DAction {
            desc: self.description,
            name: self.name,
            action_type: self.action_type.into(),
            to: self.to,
            attrs: self.attributes,
            misc: self.misc,
        }
    }
}

#[derive(Serialize, Deserialize, Default, ApiType)]
pub struct ParamsOfStart {
    address: String,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct RegisteredDebot {
    pub debot_handle: DebotHandle,
}

pub async fn start(
    context: Arc<ClientContext>,
    params: ParamsOfStart,
    callbacks: impl BrowserCallbacks + Send + Sync + 'static,
) -> ClientResult<RegisteredDebot> {
    let mut dengine = DEngine::new_with_client(
        params.address,
        None,
        context.clone(),
        Box::new(callbacks),
    );
    dengine
        .start()
        .await
        .map_err(Error::start_failed)?;

    let handle = context.get_next_id();
    context.debots.insert(handle, Mutex::new(dengine));

    Ok(RegisteredDebot {
        debot_handle: DebotHandle(handle),
    })
}

#[derive(Serialize, Deserialize, Default, ApiType)]
pub struct ParamsOfFetch {
    pub address: String,
}

pub async fn fetch(
    context: Arc<ClientContext>,
    params: ParamsOfFetch,
    callbacks: impl BrowserCallbacks + Send + Sync + 'static,
) -> ClientResult<RegisteredDebot> {
    let mut dengine = DEngine::new_with_client(
        params.address,
        None,
        context.clone(),
        Box::new(callbacks),
    );
    dengine
        .fetch()
        .await
        .map_err(Error::fetch_failed)?;

    let handle = context.get_next_id();
    context.debots.insert(handle, Mutex::new(dengine));

    Ok(RegisteredDebot {
        debot_handle: DebotHandle(handle),
    })
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ParamsOfExecute {
    pub debot_handle: DebotHandle,
    pub action: DebotAction,
}

#[api_function]
pub async fn execute(
    context: Arc<ClientContext>,
    params: ParamsOfExecute,
) -> ClientResult<()> {
    let mutex = context.debots.get(&params.debot_handle.0)
        .ok_or(Error::invalid_handle(params.debot_handle.0))?;
    let mut dengine = mutex.1.lock().await;
    dengine
        .execute_action(&params.action.into())
        .await
        .map_err(Error::execute_failed)
}

#[api_function]
pub fn remove(
    context: Arc<ClientContext>,
    params: RegisteredDebot,
) -> ClientResult<()> {
    context.debots.remove(&params.debot_handle.0)
        .ok_or(Error::invalid_handle(params.debot_handle.0))?;
    Ok(())
}
