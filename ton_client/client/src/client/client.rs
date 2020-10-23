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

#[cfg(not(feature = "wasm"))]
use super::std_client_env::ClientEnv;
#[cfg(feature = "wasm")]
use super::wasm_client_env::ClientEnv;

use super::Error;

pub struct ClientContext {
    pub(crate) client: Option<NodeClient>,
    pub(crate) config: ClientConfig,
    pub(crate) env: Arc<ClientEnv>,
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
            if config.network.out_of_sync_threshold
                > config.abi.message_expiration_timeout as i64 / 2
            {
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

        Ok(Self {
            client,
            config,
            env,
        })
    }
}

fn default_mnemonic_dictionary() -> u8 {
    1
}

fn default_mnemonic_word_count() -> u8 {
    12
}

fn default_hdkey_derivation_path() -> String {
    "m/44'/396'/0'/0/0".into()
}

fn default_hdkey_compliant() -> bool {
    true
}

#[derive(Deserialize, Debug, Clone, ApiType)]
pub struct CryptoConfig {
    #[serde(default = "default_mnemonic_dictionary")]
    pub mnemonic_dictionary: u8,
    #[serde(default = "default_mnemonic_word_count")]
    pub mnemonic_word_count: u8,
    #[serde(default = "default_hdkey_derivation_path")]
    pub hdkey_derivation_path: String,
    #[serde(default = "default_hdkey_compliant")]
    pub hdkey_compliant: bool,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            mnemonic_dictionary: default_mnemonic_dictionary(),
            mnemonic_word_count: default_mnemonic_word_count(),
            hdkey_derivation_path: default_hdkey_derivation_path(),
            hdkey_compliant: default_hdkey_compliant(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, ApiType)]
pub struct ClientConfig {
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub crypto: CryptoConfig,
    #[serde(default)]
    pub abi: AbiConfig,
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
