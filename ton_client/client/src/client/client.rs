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

#[cfg(not(target_arch = "wasm32"))]
use super::std_client_env::ClientEnvImpl;
#[cfg(target_arch = "wasm32")]
use super::wasm_client_env::ClientEnvImpl;

use super::Error;

pub struct ClientContext {
    pub(crate) client: Option<NodeClient>,
    pub(crate) config: InternalClientConfig,
    pub(crate) env: Arc<ClientEnvImpl>,
}

impl ClientContext {
    pub(crate) fn get_client(&self) -> ClientResult<&NodeClient> {
        self.client.as_ref().ok_or(Error::net_module_not_init())
    }

    pub fn new(config: Option<ClientConfig>) -> ClientResult<ClientContext> {
        let config: InternalClientConfig = config.unwrap_or_default().into();

        let env = Arc::new(super::ClientEnvImpl::new()?);

        let client = if let Some(net_config) = &config.network {
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
            let client = NodeClient::new(net_config.clone(), env.clone());
            // let sdk_config = ton_sdk::NetworkConfig {
            //     access_key: net_config.access_key.clone(),
            //     message_processing_timeout: net_config.message_processing_timeout,
            //     message_retries_count: net_config.message_retries_count,
            //     out_of_sync_threshold: net_config.out_of_sync_threshold,
            //     server_address: net_config.server_address.clone(),
            //     wait_for_timeout: net_config.wait_for_timeout,
            // };
            // let sdk_client = ton_sdk::NodeClient::new(sdk_config);
            Some(client)
        } else {
            None
        };

        Ok(Self {
            client,
            config,
            env,
        })
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct CryptoConfig {
    pub fish_param: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Default)]
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
