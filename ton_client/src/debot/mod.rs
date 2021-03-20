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
mod base64_interface;
mod hex_interface;
mod browser;
mod calltype;
mod context;
mod debot_abi;
mod dengine;
mod dinterface;
mod errors;
mod helpers;
mod info;
mod msg_interface;
mod routines;
mod run_output;
mod sdk_interface;
#[cfg(test)]
mod tests;

pub use action::DAction;
pub use browser::BrowserCallbacks;
pub use context::{DContext, STATE_EXIT, STATE_ZERO};
pub use dengine::DEngine;
pub use dinterface::{DebotInterface, DebotInterfaceExecutor, InterfaceResult};
pub use errors::{Error, ErrorCode};
pub use info::DeBotInfo;
use crate::error::ClientResult;
use crate::ClientContext;
use std::sync::Arc;
use tokio::sync::Mutex;

pub const DEBOT_WC: i8 = -31; // 0xDB

type TonClient = Arc<ClientContext>;
type JsonValue = serde_json::Value;

/// [UNSTABLE](UNSTABLE.md) Handle of registered in SDK debot
#[derive(Serialize, Deserialize, Default, ApiType, Clone)]
pub struct DebotHandle(u32);

/// [UNSTABLE](UNSTABLE.md) Describes a debot action in a Debot Context.
#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default, PartialEq)]
pub struct DebotAction {
    /// A short action description. Should be used by Debot Browser as name of
    /// menu item.
    pub description: String,
    /// Depends on action type. Can be a debot function name or a print string
    /// (for Print Action).
    pub name: String,
    /// Action type.
    pub action_type: u8,
    /// ID of debot context to switch after action execution.
    pub to: u8,
    /// Action attributes. In the form of "param=value,flag".
    /// attribute example: instant, args, fargs, sign.
    pub attributes: String,
    /// Some internal action data. Used by debot only.
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

/// [UNSTABLE](UNSTABLE.md) Parameters to start debot.
#[derive(Serialize, Deserialize, Default, ApiType)]
pub struct ParamsOfStart {
    /// Debot handle which references an instance of debot engine.
    debot_handle: DebotHandle,
}

/// [UNSTABLE](UNSTABLE.md) Starts an instance of debot.
///
/// Downloads debot smart contract from blockchain and switches it to
/// context zero.
/// Returns a debot handle which can be used later in `execute` function.
/// This function must be used by Debot Browser to start a dialog with debot.
/// While the function is executing, several Browser Callbacks can be called,
/// since the debot tries to display all actions from the context 0 to the user.
///
/// # Remarks
/// `start` is equivalent to `fetch` + switch to context 0.
///
/// When the debot starts SDK registers `BrowserCallbacks` AppObject.
/// Therefore when `debote.remove` is called the debot is being deleted and the callback is called
/// with `finish`=`true` which indicates that it will never be used again.
pub async fn start(
    context: Arc<ClientContext>,
    params: ParamsOfStart,
) -> ClientResult<()> {
    let mutex = context
        .debots
        .get(&params.debot_handle.0)
        .ok_or(Error::invalid_handle(params.debot_handle.0))?;
    let mut dengine = mutex.1.lock().await;
    dengine.start().await.map_err(Error::start_failed)
}

/// [UNSTABLE](UNSTABLE.md) Parameters to fetch debot.
#[derive(Serialize, Deserialize, Default, ApiType)]
pub struct ParamsOfFetch {
    /// Debot smart contract address
    pub address: String,
}

/// [UNSTABLE](UNSTABLE.md)
#[derive(Serialize, Deserialize, Default, ApiType)]
struct ResultOfFetch {
    /// Debot metadata,
    pub debot_info: DeBotInfo,
}

/// [UNSTABLE](UNSTABLE.md) Fetches debot from blockchain.
///
/// Downloads debot smart contract (code and data) from blockchain and creates
/// an instance of Debot Engine for it.
///
/// # Remarks
/// It does not switch debot to context 0. Browser Callbacks are not called.
pub async fn fetch(
    context: Arc<ClientContext>,
    params: ParamsOfFetch,
) -> ClientResult<ResultOfFetch>
    let mut dengine =
        DEngine::new_with_client(params.address, None, context.clone(), Arc::new(callbacks));
    let debot_info = dengine.fetch().await.map_err(Error::fetch_failed)?;

    Ok(ResultOfFetch { debot_info })
}

/// [UNSTABLE](UNSTABLE.md) Parameters to fetch debot.
#[derive(Serialize, Deserialize, Default, ApiType)]
pub struct ParamsOfInit {
    /// Debot smart contract address
    pub address: String,
}

/// [UNSTABLE](UNSTABLE.md) Structure for storing debot handle returned from `start` and `fetch` functions.
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct RegisteredDebot {
    /// Debot handle which references an instance of debot engine.
    pub debot_handle: DebotHandle,
    /// Debot abi as json string.
    pub debot_abi: String,

    pub debot_info: DeBotInfo,
}

/// [UNSTABLE](UNSTABLE.md) Fetches debot from blockchain.
///
/// Downloads debot smart contract (code and data) from blockchain and creates
/// an instance of Debot Engine for it.
///
/// # Remarks
/// It does not switch debot to context 0. Browser Callbacks are not called.
pub async fn init(
    context: Arc<ClientContext>,
    params: ParamsOfInit,
    callbacks: impl BrowserCallbacks + Send + Sync + 'static,
) -> ClientResult<RegisteredDebot>
    let mut dengine =
        DEngine::new_with_client(params.address, None, context.clone(), Arc::new(callbacks));
    let debot_info = dengine.fetch().await.map_err(Error::fetch_failed)?;

    let handle = context.get_next_id();
    context.debots.insert(handle, Mutex::new(dengine));

    Ok(RegisteredDebot { debot_handle: handle, debot_info, debot_abi: debot_info.dabi })
}

/// [UNSTABLE](UNSTABLE.md) Parameters for executing debot action.
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfExecute {
    /// Debot handle which references an instance of debot engine.
    pub debot_handle: DebotHandle,
    /// Debot Action that must be executed.
    pub action: DebotAction,
}

/// [UNSTABLE](UNSTABLE.md) Executes debot action.
///
/// Calls debot engine referenced by debot handle to execute input action.
/// Calls Debot Browser Callbacks if needed.
///
/// # Remarks
/// Chain of actions can be executed if input action generates a list of subactions.
#[api_function]
pub async fn execute(context: Arc<ClientContext>, params: ParamsOfExecute) -> ClientResult<()> {
    let mutex = context
        .debots
        .get(&params.debot_handle.0)
        .ok_or(Error::invalid_handle(params.debot_handle.0))?;
    let mut dengine = mutex.1.lock().await;
    dengine
        .execute_action(&params.action.into())
        .await
        .map_err(Error::execute_failed)
}

/// [UNSTABLE](UNSTABLE.md)
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfremove {
    /// Debot handle which references an instance of debot engine.
    pub debot_handle: DebotHandle,
}

/// [UNSTABLE](UNSTABLE.md) Destroys debot handle.
///
/// Removes handle from Client Context and drops debot engine referenced by that handle.
#[api_function]
pub fn remove(context: Arc<ClientContext>, params: ParamsOfremove) -> ClientResult<()> {
    context.debots.remove(&params.debot_handle.0);
    Ok(())
}

/// [UNSTABLE](UNSTABLE.md) Parameters of `send` function.
#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfSend {
    /// Debot handle which references an instance of debot engine.
    pub debot_handle: DebotHandle,
    /// BOC of internal message to debot encoded in base64 format.
    pub message: String,
}

/// [UNSTABLE](UNSTABLE.md) Sends message to Debot.
///
/// Used by Debot Browser to send response on Dinterface call or from other Debots.
#[api_function]
pub async fn send(context: Arc<ClientContext>, params: ParamsOfSend) -> ClientResult<()> {
    let mutex = context
        .debots
        .get(&params.debot_handle.0)
        .ok_or(Error::invalid_handle(params.debot_handle.0))?;
    let mut dengine = mutex.1.lock().await;
    dengine
        .send(params.message)
        .await
}
