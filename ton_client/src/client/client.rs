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

use lockfree::map::Map as LockfreeMap;
use serde::{Deserialize, Deserializer};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::collections::HashMap;
use tokio::sync::{oneshot, Mutex};

use super::{AppRequestParams, Error, RequestResult};
use crate::error::ClientResult;
use crate::abi::AbiConfig;
use crate::crypto::CryptoConfig;
use crate::crypto::boxes::SigningBox;
use crate::json_interface::request::Request;
use crate::json_interface::interop::ResponseType;
use crate::net::{NetworkConfig, NodeClient};

#[cfg(not(feature = "wasm"))]
use super::std_client_env::ClientEnv;
#[cfg(feature = "wasm")]
use super::wasm_client_env::ClientEnv;

#[derive(Default)]
pub struct Boxes {
    pub(crate) signing_boxes: LockfreeMap<u32, Box<dyn SigningBox + Send + Sync>>,
}

pub struct ClientContext {
    pub(crate) client: Option<NodeClient>,
    pub(crate) config: ClientConfig,
    pub(crate) env: Arc<ClientEnv>,
    pub(crate) boxes: Boxes,
    pub(crate) app_requests: Mutex<HashMap<u32, oneshot::Sender<RequestResult>>>,

    next_id: AtomicU32,
}

impl ClientContext {
    pub(crate) fn get_client(&self) -> ClientResult<&NodeClient> {
        self.client.as_ref().ok_or(Error::net_module_not_init())
    }

    pub async fn set_timer(&self, ms: u64) -> ClientResult<()> {
        self.env.set_timer(ms).await
    }

    pub fn new(config: ClientConfig) -> ClientResult<ClientContext> {
        let env = Arc::new(ClientEnv::new()?);

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

        Ok(Self {
            client,
            config,
            env,
            boxes: Default::default(),
            app_requests: Mutex::new(HashMap::new()),
            next_id: AtomicU32::new(1),
        })
    }

    pub(crate) fn get_next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) async fn app_request<R: serde::de::DeserializeOwned>(
        &self,
        callback: &Request,
        object_ref: String,
        params: impl serde::Serialize,
    ) -> ClientResult<R> {
        let id = self.get_next_id();
        let (sender, receiver) = oneshot::channel();
        self.app_requests
            .lock()
            .await
            .insert(id, sender);
        
        callback.response(
            AppRequestParams {
                app_request_id: id,
                object_ref,
                request_data: params
            },
            ResponseType::AppRequest as u32);
        let result = receiver
            .await
            .map_err(|err| Error::can_not_receive_request_result(err))?;

        match result {
            RequestResult::Error { text } => Err(Error::app_request_error(&text)),
            RequestResult::Ok { result } => {
                serde_json::from_value(result)
                    .map_err(|err| Error::can_not_parse_request_result(err))
            }
        }
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
