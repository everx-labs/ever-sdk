/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::error::ClientResult;
use std::sync::Arc;
use ton_sdk::AbiConfig;

use crate::net::{NetworkConfig, NodeClient};

use super::std_client_env::StdClientEnv;
use super::{ClientEnv, Error};

pub struct ClientContext {
    #[cfg(feature = "node_interaction")]
    pub(crate) client: Option<NodeClient>,
    #[cfg(feature = "node_interaction")]
    _async_runtime: Option<tokio::runtime::Runtime>,
    #[cfg(feature = "node_interaction")]
    pub(crate) async_runtime_handle: tokio::runtime::Handle,
    pub(crate) config: InternalClientConfig,
    pub(crate) env: Arc<dyn ClientEnv + Send + Sync>,
}

#[cfg(feature = "node_interaction")]
impl ClientContext {
    pub(crate) fn get_client(&self) -> ClientResult<&NodeClient> {
        self.client.as_ref().ok_or(Error::net_module_not_init())
    }

    #[cfg(not(feature = "node_interaction"))]
    pub fn new(config: Option<ClientConfig>) -> ClientResult<Self> {
        Ok(Self {
            config: config.unwrap_or_default().into(),
        })
    }

    #[cfg(feature = "node_interaction")]
    pub fn new(config: Option<ClientConfig>) -> ClientResult<ClientContext> {
        let config: InternalClientConfig = config.unwrap_or_default().into();
        let std_env = Arc::new(StdClientEnv::new()?);

        let (client, _) = if let Some(net_config) = &config.network {
            if net_config.out_of_sync_threshold()
                > config.abi.message_expiration_timeout() as i64 / 2
            {
                return Err(Error::invalid_config(format!(
                    r#"`out_of_sync_threshold` can not be more then `message_expiration_timeout / 2`.
`out_of_sync_threshold` = {}, `message_expiration_timeout` = {}
Note that default values are used if parameters are omitted in config"#,
                    net_config.out_of_sync_threshold(),
                    config.abi.message_expiration_timeout()
                )));
            }
            let client = NodeClient::new(net_config.clone(), std_env.clone());
            let sdk_config = ton_sdk::NetworkConfig {
                access_key: net_config.access_key.clone(),
                message_processing_timeout: net_config.message_processing_timeout,
                message_retries_count: net_config.message_retries_count,
                out_of_sync_threshold: net_config.out_of_sync_threshold,
                server_address: net_config.server_address.clone(),
                wait_for_timeout: net_config.wait_for_timeout,
            };
            let sdk_client = ton_sdk::NodeClient::new(sdk_config);
            (Some(client), Some(sdk_client))
        } else {
            (None, None)
        };

        let (async_runtime, async_runtime_handle) =
            if let Ok(existing) = tokio::runtime::Handle::try_current() {
                (None, existing)
            } else {
                let runtime = tokio::runtime::Builder::new()
                    .threaded_scheduler()
                    .enable_io()
                    .enable_time()
                    .build()
                    .map_err(|err| Error::cannot_create_runtime(err))?;
                let runtime_handle = runtime.handle().clone();
                (Some(runtime), runtime_handle)
            };

        Ok(Self {
            client,
            _async_runtime: async_runtime,
            async_runtime_handle,
            config,
            env: std_env,
        })
    }
}

#[derive(Deserialize, Debug, Default, Clone, ApiType)]
pub struct CryptoConfig {
    pub fish_param: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Default, ApiType)]
pub struct ClientConfig {
    pub network: Option<NetworkConfig>,
    pub crypto: Option<CryptoConfig>,
    pub abi: Option<AbiConfig>,
}

#[derive(Debug, Clone)]
pub struct InternalClientConfig {
    pub network: Option<NetworkConfig>,
    pub crypto: CryptoConfig,
    pub abi: AbiConfig,
}

impl From<ClientConfig> for InternalClientConfig {
    fn from(config: ClientConfig) -> Self {
        InternalClientConfig {
            network: config.network,
            crypto: config.crypto.unwrap_or_default(),
            abi: config.abi.unwrap_or_default(),
        }
    }
}
