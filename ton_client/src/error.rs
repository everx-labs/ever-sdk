use crate::client::core_version;
use serde_json::Value;
use std::fmt::Display;
use chrono::TimeZone;
use crate::net;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, ApiType)]
pub struct ClientError {
    pub code: u32,
    pub message: String,
    pub data: Value,
}

pub type ClientResult<T> = Result<T, ClientError>;

#[async_trait::async_trait]
pub(crate) trait AddNetworkUrl: Sized {
    async fn add_endpoint_from_context(
        self,
        context: &crate::ClientContext,
        endpoint: &net::Endpoint,
    ) -> Self {
        if let Some(link) = &context.net.server_link {
            self.add_endpoint(link, endpoint).await
        } else {
            self
        }
    }

    async fn add_network_url(self, client: &net::ServerLink) -> Self {
        self.add_network_url_from_state(client.state().as_ref()).await
    }

    async fn add_network_url_from_context(self, client: &crate::ClientContext) -> Self {
        if let Some(client) = &client.net.server_link {
            self.add_network_url(client).await
        } else {
            self
        }
    }

    async fn add_endpoint(
        self,
        link: &net::ServerLink,
        endpoint: &net::Endpoint,
    ) -> Self;

    async fn add_network_url_from_state(self, state: &net::NetworkState) -> Self;
}

#[async_trait::async_trait]
impl<T: Send> AddNetworkUrl for ClientResult<T> {
    async fn add_endpoint(
        self,
        link: &net::ServerLink,
        endpoint: &net::Endpoint,
    ) -> Self {
        match self {
            Err(err) => {
                Err(err.add_endpoint(link, endpoint).await)
            }
            _ => self,
        }
    }

    async fn add_network_url_from_state(self, state: &net::NetworkState) -> Self {
        match self {
            Err(err) => {
                Err(err.add_network_url_from_state(state).await)
            }
            _ => self,
        }
    }
}

#[async_trait::async_trait]
impl AddNetworkUrl for ClientError {
    async fn add_endpoint(
        mut self,
        link: &net::ServerLink,
        endpoint: &net::Endpoint,
    ) -> Self {
        self.data["config_servers"] = link.config_servers().await.into();
        self.data["endpoint"] = Value::String(endpoint.query_url.clone());
        self
    }

    async fn add_network_url_from_state(mut self, state: &net::NetworkState) -> Self {
        self.data["config_servers"] = state.config_servers().await.into();
        if let Some(endpoint) = state.query_endpoint().await {
            self.data["query_url"] = endpoint.query_url.as_str().into();
            if let Some(ip_address) = &endpoint.ip_address {
                self.data["query_ip_address"] = ip_address.as_str().into();
            }
        }
        self
    }
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:#}", json!(self))
        } else {
            write!(f, "{}", self.message)
        }
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
    pub const DEBOT: isize = 800;

    pub fn new(code: u32, message: String, data: Value) -> Self {
        let mut data = data;
        data["core_version"] = Value::String(core_version());
        Self {
            code,
            message,
            data,
        }
    }

    pub fn with_code_message(code: u32, message: String) -> Self {
        Self {
            code,
            message,
            data: json!({
                "core_version": core_version(),
            }),
        }
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

    pub fn is_unauthorized(&self) -> bool {
        self.code == net::ErrorCode::Unauthorized as u32
    }
}

pub(crate) fn format_time(time: u32) -> String {
    format!(
        "{} ({})",
        chrono::Local.timestamp_opt(time as i64, 0).unwrap().to_rfc2822(),
        time
    )
}
