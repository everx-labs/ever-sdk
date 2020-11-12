use crate::crypto::KeyPair;

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub enum DebotBrowserResponse {
    Log,
    Switch,
    ShowAction,
    Input { value: String },
    LoadKey { keys: KeyPair },
    InvokeDebot,
}
