use crate::crypto::SigningBoxHandle;
use ton_block::MsgAddressInt;
use ton_types::UInt256;
use ed25519_dalek::PublicKey;
use chrono::NaiveDateTime;

/// Callbacks that are called by debot engine to communicate with Debot Browser.
#[async_trait::async_trait]
pub trait CppBrowserCallbacks {
    /// Prints text message to user.
    async fn log(&self, msg: String);

    /// Requests keys from user.
    async fn get_signing_box(&self) -> Result<SigningBoxHandle, String>;

    /// Requests input from user.
    async fn input(&self, prompt: String) -> Option<String>;
    async fn input_address(&self, prompt: String) -> Option<MsgAddressInt>;
    async fn input_uint256(&self, prompt: String) -> Option<UInt256>;
    async fn input_pubkey(&self, prompt: String) -> Option<PublicKey>;
    async fn input_tons(&self, prompt: String) -> Option<String>;
    async fn input_yes_or_no(&self, prompt: String) -> Option<bool>;
    async fn input_datetime(&self, prompt: String) -> Option<NaiveDateTime>;
    async fn input_deploy_message(&self, prompt: String) -> Option<String>;
}
