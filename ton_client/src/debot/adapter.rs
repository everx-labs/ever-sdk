use super::action::DAction;
use super::browser::BrowserCallbacks;
use super::drequest::DebotBrowserRequest;
use super::dresponse::DebotBrowserResponse;
use crate::crypto::KeyPair;
use crate::error::ClientResult;
use futures::Future;

pub struct DebotBrowserAdapter<Fut, C>
where
    Fut: Future<Output = ClientResult<DebotBrowserResponse>> + Send + Sync + 'static,
    C: Fn(DebotBrowserRequest) -> Fut + Send + Sync,
{
    app_callback: C,
    app_ref: String,
}

impl<Fut, C> DebotBrowserAdapter<Fut, C>
where
    Fut: Future<Output = ClientResult<DebotBrowserResponse>> + Send + Sync + 'static,
    C: Fn(DebotBrowserRequest) -> Fut + Send + Sync,
{
    pub fn new(app_callback: C, app_ref: String) -> Self {
        Self {
            app_callback,
            app_ref,
        }
    }
}

#[async_trait::async_trait]
impl<Fut, C> BrowserCallbacks for DebotBrowserAdapter<Fut, C>
where
    Fut: Future<Output = ClientResult<DebotBrowserResponse>> + Send + Sync + 'static,
    C: Fn(DebotBrowserRequest) -> Fut + Send + Sync,
{
    async fn log(&self, msg: String) {
        let response = (self.app_callback)(DebotBrowserRequest::Log {
            app_ref: self.app_ref.clone(),
            msg,
        })
        .await
        .map_err(|e| error!("debot browser failed to log message: {}", e));
    }

    async fn switch(&self, ctx_id: u8) {
        let response = (self.app_callback)(DebotBrowserRequest::Switch {
            app_ref: self.app_ref.clone(),
            ctx_id,
        })
        .await
        .map_err(|e| error!("debot browser failed to switch context: {}", e));
    }

    async fn show_action(&self, act: DAction) {
        let response = (self.app_callback)(DebotBrowserRequest::ShowAction {
            app_ref: self.app_ref.clone(),
            action: act,
        })
        .await
        .map_err(|e| error!("debot browser failed to show action: {}", e));
    }

    async fn input(&self, prefix: &str, value: &mut String) {
        let response = (self.app_callback)(DebotBrowserRequest::Input {
            app_ref: self.app_ref.clone(),
            prefix: prefix.to_owned(),
        })
        .await;

        *value = String::new();
        match response {
            Ok(r) => match r {
                DebotBrowserResponse::Input { value: v } => *value = v,
                _ => error!("unexpected debot browser response: {:?}", r),
            },
            Err(e) => error!("debot browser failed to show action: {}", e),
        }
    }

    async fn load_key(&self, keys: &mut KeyPair) {
        let response = (self.app_callback)(DebotBrowserRequest::LoadKey {
            app_ref: self.app_ref.clone(),
        })
        .await;

        match response {
            Ok(r) => match r {
                DebotBrowserResponse::LoadKey { keys: k } => *keys = k,
                _ => error!("unexpected debot browser response: {:?}", r),
            },
            Err(e) => error!("debot browser failed to load keys: {}", e),
        }
    }

    async fn invoke_debot(&self, debot: String, action: DAction) -> Result<(), String> {
        let response = (self.app_callback)(DebotBrowserRequest::InvokeDebot {
            app_ref: self.app_ref.clone(),
            debot_addr: debot,
            action,
        })
        .await
        .map_err(|e| {
            error!("debot browser failed to invoke debot: {}", e);
            format!("debot browser failed to invoke debot: {}", e)
        })?;
        Ok(())
    }
}
