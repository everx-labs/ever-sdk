use super::action::DAction;
use crate::crypto::KeyPair;

#[async_trait::async_trait]
pub trait BrowserCallbacks {
    /// Debot sends text message to user.
    async fn log(&self, msg: String);
    /// Debot is switched to another context.
    async fn switch(&self, ctx_id: u8);
    // Dengine calls this callback after `switch` callback for every action in context
    async fn show_action(&self, act: DAction);
    // Debot engine asks user to enter argument for an action.
    async fn input(&self, prefix: &str, value: &mut String);

    async fn load_key(&self, keys: &mut KeyPair);

    async fn invoke_debot(&self, debot: String, action: DAction) -> Result<(), String>;
}
