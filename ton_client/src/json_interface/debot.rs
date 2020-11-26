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
 use crate::crypto::SigningBoxHandle;

/// **UNSTABLE API.** Returning values from Debot Browser callbacks.
#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
#[serde(tag="type")]
pub enum ResultOfAppDebotBrowser {
    /// Result of user input.
    Input {
        /// String entered by user.
        value: String
    },
    /// Result of getting signing box.
    GetSigningBox { 
        /// Signing box for signing data requested by debot engine. Signing box is owned and disposed by debot engine
        signing_box: SigningBoxHandle
    },
    /// Result of debot invoking.
    InvokeDebot,
}

/// **UNSTABLE API.** Debot Browser callbacks
/// 
/// Called by debot engine to communicate with debot browser.
#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
#[serde(tag="type")]
pub enum ParamsOfAppDebotBrowser {
    /// Print message to user. 
    Log {
        /// A string that must be printed to user.
        msg: String
    },
    /// Switch debot to another context (menu).
    Switch {
        /// Debot context ID to which debot is switched.
        context_id: u8
    },
    /// Show action to the user.
    /// Called after `switch` for each action in context.
    ShowAction {
        /// Debot action that must be shown to user as menu item.
        /// At least `description` property must be shown from [DebotAction] structure.
        action: DebotAction
    },
    /// Request user input. 
    Input {
        /// A prompt string that must be printed to user before input request.
        prompt: String
    },
    /// Get signing box to sign data. Signing box returned is owned and disposed by debot engine
    GetSigningBox,
    /// Execute action of another debot.
    InvokeDebot {
        /// Address of debot in blockchain.
        debot_addr: String,
        /// Debot action to execute.
        action: DebotAction
    },
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
 
     async fn input(&self, prompt: &str, value: &mut String) {
         let response = self.app_object.call(ParamsOfAppDebotBrowser::Input {
                 prompt: prompt.to_owned(),
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
 
     async fn get_signing_box(&self) -> Result<SigningBoxHandle, String> {
         let response = self.app_object.call(ParamsOfAppDebotBrowser::GetSigningBox)
             .await
             .map_err(|err| format!("debot browser failed to load keys: {}", err))?;
 
        match response {
            ResultOfAppDebotBrowser::GetSigningBox { signing_box } => Ok(signing_box),
            _ => Err(crate::client::Error::unexpected_callback_response(
                "GetSigningBox", response).to_string()),
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

/// **UNSTABLE API.** Starts an instance of debot.
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
#[api_function]
pub(crate) async fn start(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfStart,
    app_object: AppObject<ParamsOfAppDebotBrowser, ResultOfAppDebotBrowser>,
) -> ClientResult<RegisteredDebot> {
    let browser_callbacks = DebotBrowserAdapter::new(app_object);
    crate::debot::start(context, params, browser_callbacks).await
}

/// **UNSTABLE API.** Fetches debot from blockchain.
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

