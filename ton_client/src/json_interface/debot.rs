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
 
#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub enum ResultOfAppDebotBrowser {
    Input { value: String },
    LoadKey { keys: KeyPair },
    InvokeDebot,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub enum ParamsOfAppDebotBrowser {
    Log { msg: String },
    Switch { context_id: u8 },
    ShowAction { action: DebotAction },
    Input { prefix: String },
    LoadKey,
    InvokeDebot { debot_addr: String, action: DebotAction },
}
 
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

#[api_function]
pub(crate) async fn start(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfStart,
    app_object: AppObject<ParamsOfAppDebotBrowser, ResultOfAppDebotBrowser>,
) -> ClientResult<RegisteredDebot> {
    let browser_callbacks = DebotBrowserAdapter::new(app_object);
    crate::debot::start(context, params, browser_callbacks).await
}

#[api_function]
pub(crate) async fn fetch(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfFetch,
    app_object: AppObject<ParamsOfAppDebotBrowser, ResultOfAppDebotBrowser>,
) -> ClientResult<RegisteredDebot> {
    let browser_callbacks = DebotBrowserAdapter::new(app_object);
    crate::debot::fetch(context, params, browser_callbacks).await
}

