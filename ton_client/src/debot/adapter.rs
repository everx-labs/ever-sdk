use super::action::DAction;
use super::browser::BrowserCallbacks;
use super::ParamsOfAppDebotBrowser;
use super::ResultOfAppDebotBrowser;
use crate::client::AppObject;
use crate::crypto::KeyPair;

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
        self.app_object.notify(ParamsOfAppDebotBrowser::Switch { ctx_id });
    }

    async fn show_action(&self, act: DAction) {
        self.app_object.notify(ParamsOfAppDebotBrowser::ShowAction { action: act });
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
            action,
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
