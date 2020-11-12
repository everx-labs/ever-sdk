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
use crate::debot::DebotContext;

use crate::net::{NetworkConfig, NodeClient};

#[cfg(not(feature = "wasm"))]
use super::std_client_env::ClientEnv;
#[cfg(feature = "wasm")]
use super::wasm_client_env::ClientEnv;

use super::Error;
use crate::abi::AbiConfig;
use crate::crypto::CryptoConfig;
use serde::{Deserialize, Deserializer};

pub struct ClientContext {
    pub(crate) client: Option<NodeClient>,
    pub(crate) config: ClientConfig,
    pub(crate) env: Arc<ClientEnv>,
    pub(crate) debot_ctx: tokio::sync::RwLock<DebotContext>
}

impl ClientContext {
    pub(crate) fn get_client(&self) -> ClientResult<&NodeClient> {
        self.client.as_ref().ok_or(Error::net_module_not_init())
    }

    pub async fn set_timer(&self, ms: u64) -> ClientResult<()> {
        self.env.set_timer(ms).await
    }

    pub fn new(config: ClientConfig) -> ClientResult<ClientContext> {
        let env = Arc::new(super::ClientEnv::new()?);

        let client = if !config.network.server_address.is_empty() {
            if config.network.out_of_sync_threshold > config.abi.message_expiration_timeout / 2 {
                return Err(Error::invalid_config(format!(
                    r#"`out_of_sync_threshold` can not be more then `message_expiration_timeout / 2`.
`out_of_sync_threshold` = {}, `message_expiration_timeout` = {}
Note that default values are used if parameters are omitted in config"#,
                    config.network.out_of_sync_threshold, config.abi.message_expiration_timeout
                )));
            }
            Some(NodeClient::new(config.network.clone(), env.clone()))
        } else {
            None
        };

        let debot_ctx = tokio::sync::RwLock::new(DebotContext::new());

        Ok(Self {
            client,
            config,
            env,
            debot_ctx,
        })
    }
}

#[derive(Deserialize, Debug, Clone, ApiType)]
pub struct ClientConfig {
    #[serde(default, deserialize_with = "deserialize_network_config")]
    pub network: NetworkConfig,
    #[serde(default, deserialize_with = "deserialize_crypto_config")]
    pub crypto: CryptoConfig,
    #[serde(default, deserialize_with = "deserialize_abi_config")]
    pub abi: AbiConfig,
}

fn deserialize_network_config<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<NetworkConfig, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(Default::default()))
}

fn deserialize_crypto_config<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<CryptoConfig, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(Default::default()))
}

fn deserialize_abi_config<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<AbiConfig, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(Default::default()))
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            network: Default::default(),
            crypto: Default::default(),
            abi: Default::default(),
        }
    }
}
