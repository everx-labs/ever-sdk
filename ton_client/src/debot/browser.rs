use super::action::DAction;
use super::DebotActivity;
use crate::crypto::SigningBoxHandle;
use crate::error::ClientResult;

/// Callbacks that are called by debot engine to communicate with Debot Browser.
#[async_trait::async_trait]
pub trait BrowserCallbacks {
    /// Prints text message to user.
    async fn log(&self, msg: String);
    /// Notify that debot is switched to another context.
    async fn switch(&self, ctx_id: u8);
    /// Notify that all actions are shown to user and switch to conetxt is completed.
    async fn switch_completed(&self);
    /// Show action to the user as menu item.
    /// Called after `switch` callback for every action in context.
    async fn show_action(&self, act: DAction);
    /// Requests input from user.
    async fn input(&self, prompt: &str, value: &mut String);
    /// Requests keys from user.
    async fn get_signing_box(&self) -> Result<SigningBoxHandle, String>;
    /// Executes action of another debot.
    async fn invoke_debot(&self, debot: String, action: DAction) -> Result<(), String>;

    /// Sends message with debot interface call to Browser.
    /// Message parameter is a BoC encoded as Base64.
    async fn send(&self, message: String);

    /// Requests permission to execute DeBot operation
    /// (e.g. sending messages to blockchain).
    async fn approve(&self, activity: DebotActivity) -> ClientResult<bool>;
}
