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
 *
 */

 use crate::client::{AppObject, ClientContext};
 use crate::error::ClientResult;
 use crate::debot::{DAction, DebotAction, BrowserCallbacks, ParamsOfFetch, ParamsOfStart, RegisteredDebot};
 use crate::crypto::KeyPair;

/// Returning values from Debot Browser callbacks.
#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub enum ResultOfAppDebotBrowser {
    /// Result of `input` callback.
    /// `value` - string entered by user.
    Input { value: String },
    /// Result of `load_key` callback.
    /// `keys` - keypair that browser asked from user.
    LoadKey { keys: KeyPair },
    InvokeDebot,
}

/// Debot Browser callbacks
/// 
/// Called by debot engine to communicate with debot browser.
#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub enum ParamsOfAppDebotBrowser {
    /// `log` callback. Prints message to user. 
    /// `msg` is a string that must be printed to user.
    Log { msg: String },
    /// `switch` callback. Switch debot to another context (menu). 
    /// `context_id` - debot context id to which debot is switched.
    Switch { context_id: u8 },
    /// `show_action` callback. Called after `switch` for each action in context.
    /// Shows action to the user.
    /// `action` - debot action that must be shown to user as menu item.
    /// At least `desc` property must be shown from [DebotAction] structure.
    ShowAction { action: DebotAction },
    /// `input` callback. Request from debot to input data. 
    /// `prefix` - a promt string that must be printed to user before input.  
    Input { prefix: String },
    /// `load_key` callback. Request from debot to load keypair.
    LoadKey,
    /// `invoke_debot` callback. Requests to execute action of another debot.
    /// `debot_addr` - address of debot in blockchain.
    /// `action` - debot action to execute.
    InvokeDebot { debot_addr: String, action: DebotAction },
}
 
/// Wrapper for native Debot Browser callbacks.
/// 
/// Adapter between SDK application and low level debot interface.
pub(crate) struct DebotBrowserAdapter {
    app_object: AppObject<ParamsOfAppDebotBrowser, ResultOfAppDebotBrowser>,
}
 
impl DebotBrowserAdapter {
    pub fn new(app_object: AppObject<ParamsOfAppDebotBrowser, ResultOfAppDebotBrowser>) -> Self {
        Self { app_object }
    }
}
 
 #[async_trait::async_trait]
 impl BrowserCallbacks for DebotBrowserAdapter {
     
     async fn log(&self, msg: String) {
         self.app_object.notify(ParamsOfAppDebotBrowser::Log { msg });
     }
 
     async fn switch(&self, ctx_id: u8) {
         self.app_object.notify(ParamsOfAppDebotBrowser::Switch { context_id: ctx_id });
     }
 
     async fn show_action(&self, act: DAction) {
         self.app_object.notify(ParamsOfAppDebotBrowser::ShowAction { action: act.into() });
     }
 
     async fn input(&self, prefix: &str, value: &mut String) {
         let response = self.app_object.call(ParamsOfAppDebotBrowser::Input {
                 prefix: prefix.to_owned(),
             })
             .await;
         match response {
             Ok(r) => match r {
                 ResultOfAppDebotBrowser::Input { value: v } => *value = v,
                 _ => error!("unexpected debot browser response: {:?}", r),
             },
             Err(e) => error!("debot browser failed to show action: {}", e),
         }
     }
 
     async fn load_key(&self, keys: &mut KeyPair) {
         let response = self.app_object.call(ParamsOfAppDebotBrowser::LoadKey)
             .await;
 
         match response {
             Ok(r) => match r {
                 ResultOfAppDebotBrowser::LoadKey { keys: k } => *keys = k,
                 _ => error!("unexpected debot browser response: {:?}", r),
             },
             Err(e) => error!("debot browser failed to load keys: {}", e),
         }
     }
 
     async fn invoke_debot(&self, debot: String, action: DAction) -> Result<(), String> {
         let response = self.app_object.call(ParamsOfAppDebotBrowser::InvokeDebot {
             debot_addr: debot,
             action: action.into(),
         })
         .await
         .map_err(|e| {
             error!("debot browser failed to invoke debot: {}", e);
             format!("debot browser failed to invoke debot: {}", e)
         })?;
 
         match response {
             ResultOfAppDebotBrowser::InvokeDebot => Ok(()),
             _ => {
                 error!("unexpected debot browser response: {:?}", response);
                 Err(format!("unexpected debot browser response: {:?}", response))
             },
         }
     }
 }

/// Starts an instance of debot.
/// 
/// Downloads debot smart contract from blockchain and switches it to
/// context zero.
/// Returns a debot handle which can be used later in [execute] function.
/// This function must be used by Debot Browser to start a dialog with debot.
/// While the function is executing, several Browser Callbacks can be called,
/// since the debot tries to display all actions from the context 0 to the user.
/// 
/// # Remarks
/// [start] is equivalent to [fetch] + switch to context 0.
#[api_function]
pub(crate) async fn start(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfStart,
    app_object: AppObject<ParamsOfAppDebotBrowser, ResultOfAppDebotBrowser>,
) -> ClientResult<RegisteredDebot> {
    let browser_callbacks = DebotBrowserAdapter::new(app_object);
    crate::debot::start(context, params, browser_callbacks).await
}

/// Fetches debot from blockchain.
/// 
/// Downloads debot smart contract (code and data) from blockchain and creates 
/// an instance of Debot Engine for it.
/// 
/// # Remarks
/// It does not switch debot to context 0. Browser Callbacks are not called.
#[api_function]
pub(crate) async fn fetch(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfFetch,
    app_object: AppObject<ParamsOfAppDebotBrowser, ResultOfAppDebotBrowser>,
) -> ClientResult<RegisteredDebot> {
    let browser_callbacks = DebotBrowserAdapter::new(app_object);
    crate::debot::fetch(context, params, browser_callbacks).await
}

