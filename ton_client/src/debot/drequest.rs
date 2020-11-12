use super::action::DAction;

#[derive(Serialize, Deserialize, Clone, ApiType)]
pub enum DebotBrowserRequest {
    Log {
        app_ref: String,
        msg: String,
    },
    Switch {
        app_ref: String,
        ctx_id: u8,
    },
    ShowAction {
        app_ref: String,
        action: DAction,
    },
    Input {
        app_ref: String,
        prefix: String,
    },
    LoadKey {
        app_ref: String,
    },
    InvokeDebot {
        app_ref: String,
        debot_addr: String,
        action: DAction,
    },
}
