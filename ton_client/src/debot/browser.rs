use super::action::DAction;
use crate::crypto::KeyPair;

/// Callbacks that are called by debot engine to communicate with Debot Browser.
#[async_trait::async_trait]
pub trait BrowserCallbacks {
    /// Prints text message to user.
    async fn log(&self, msg: String);
    /// Notify that debot is switched to another context.
    async fn switch(&self, ctx_id: u8);
    /// Show action to the user as menu item.
    /// Called after `switch` callback for every action in context.
    async fn show_action(&self, act: DAction);
    /// Requests input from user.
    async fn input(&self, prompt: &str, value: &mut String);
    /// Requests keys from user.
    async fn load_key(&self, keys: &mut KeyPair);
    /// Executes action of another debot.
    async fn invoke_debot(&self, debot: String, action: DAction) -> Result<(), String>;
}
