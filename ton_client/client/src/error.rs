use serde_json::Value;
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, ApiType)]
pub struct ClientError {
    pub code: isize,
    pub message: String,
    pub data: serde_json::Value,
}

pub type ClientResult<T> = Result<T, ClientError>;

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ClientError {}

impl ClientError {
    pub const CLIENT: isize = 0;
    pub const CRYPTO: isize = 100;
    pub const BOC: isize = 200;
    pub const ABI: isize = 300;
    pub const TVM: isize = 400;
    pub const PROCESSING: isize = 500;
    pub const NET: isize = 600;
    pub const UTILS: isize = 700;

    pub fn new(code: isize, message: String, data: Value) -> Self {
        let mut data = data;
        data["core_version"] = Value::String(env!("CARGO_PKG_VERSION").to_owned());
        Self {
            code,
            message,
            data,
        }
    }

    pub fn with_code_message(code: isize, message: String) -> Self {
        Self {
            code,
            message,
            data: json!({
                "core_version": env!("CARGO_PKG_VERSION").to_owned(),
            }),
        }
    }

    pub(crate) fn add_network_url(mut self, client: &crate::net::NodeClient) -> ClientError {
        self.data["config_server"] = client.config_server().into();

        if let Some(url) = client.query_url() {
            self.data["query_url"] = url.into();
        }

        self
    }

    pub fn add_function(mut self, function: Option<&str>) -> ClientError {
        if let Some(function) = function {
            self.data["function_name"] = function.into();
        }

        self
    }

    pub fn add_address(mut self, address: &ton_block::MsgAddressInt) -> ClientError {
        self.data["account_address"] = address.to_string().into();
        self
    }
}
