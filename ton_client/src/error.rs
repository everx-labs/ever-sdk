use crate::client::core_version;
use serde_json::Value;
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, ApiType)]
pub struct ClientError {
    pub code: u32,
    pub message: String,
    pub data: serde_json::Value,
}

pub type ClientResult<T> = Result<T, ClientError>;

#[async_trait::async_trait]
pub(crate) trait AddNetworkUrl {
    async fn add_endpoint(
        self,
        link: &crate::net::ServerLink,
        endpoint: &crate::net::Endpoint,
    ) -> Self;
    async fn add_endpoint_from_context(
        self,
        context: &crate::ClientContext,
        endpoint: &crate::net::Endpoint,
    ) -> Self;
    async fn add_network_url(self, link: &crate::net::ServerLink) -> Self;
    async fn add_network_url_from_context(self, client: &crate::ClientContext) -> Self;
}

#[async_trait::async_trait]
impl<T: Send> AddNetworkUrl for ClientResult<T> {
    async fn add_endpoint(
        self,
        link: &crate::net::ServerLink,
        endpoint: &crate::net::Endpoint,
    ) -> Self {
        match self {
            Err(mut err) => {
                err.data["config_servers"] = link.config_servers().await.into();
                err.data["endpoint"] = Value::String(endpoint.query_url.clone());
                Err(err)
            }
            _ => self,
        }
    }

    async fn add_endpoint_from_context(
        self,
        client: &crate::ClientContext,
        endpoint: &crate::net::Endpoint,
    ) -> Self {
        if let Some(link) = &client.net.server_link {
            self.add_endpoint(link, endpoint).await
        } else {
            self
        }
    }
    async fn add_network_url(self, client: &crate::net::ServerLink) -> Self {
        match self {
            Err(mut err) => {
                err.data["config_servers"] = client.config_servers().await.into();
                if let Some(endpoint) = client.query_endpoint().await {
                    err.data["query_url"] = endpoint.query_url.as_str().into();
                    if let Some(ip_address) = &endpoint.ip_address {
                        err.data["query_ip_address"] = ip_address.as_str().into();
                    }
                }
                Err(err)
            }
            _ => self,
        }
    }

    async fn add_network_url_from_context(self, client: &crate::ClientContext) -> Self {
        if let Some(client) = &client.net.server_link {
            self.add_network_url(client).await
        } else {
            self
        }
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
}
