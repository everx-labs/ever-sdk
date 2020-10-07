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

use crate::error::{ApiError, ApiResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use ton_sdk::AbiConfig;

use crate::net::{NetworkConfig, NodeClient};

use super::std_client_env::StdClientEnv;
use super::{ClientEnv, Error};
use serde::de::DeserializeOwned;

lazy_static! {
    static ref CLIENT: Mutex<Client> = Mutex::new(Client::new());
}

pub type ContextHandle = u32;

pub struct ClientContext {
    pub(crate) handle: ContextHandle,
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
    pub(crate) fn get_client(&self) -> ApiResult<&NodeClient> {
        self.client.as_ref().ok_or(Error::net_module_not_init())
    }
}

pub struct Client {
    next_context_handle: ContextHandle,
    contexts: HashMap<ContextHandle, Arc<ClientContext>>,
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

pub(crate) fn parse_params<P: DeserializeOwned>(params_json: &str) -> ApiResult<P> {
    serde_json::from_str(params_json).map_err(|err| ApiError::invalid_params(params_json, err))
}

#[cfg(feature = "node_interaction")]
pub fn create_context(handle: ContextHandle, config: ClientConfig) -> ApiResult<ClientContext> {
    let config: InternalClientConfig = config.into();

    let std_env = Arc::new(StdClientEnv::new()?);

    let (client, _) = if let Some(net_config) = &config.network {
        if net_config.out_of_sync_threshold() > config.abi.message_expiration_timeout() as i64 / 2 {
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

    Ok(ClientContext {
        handle,
        client,
        _async_runtime: async_runtime,
        async_runtime_handle,
        config,
        env: std_env,
    })
}

impl Client {
    fn new() -> Self {
        Self {
            next_context_handle: 1,
            contexts: HashMap::new(),
        }
    }

    pub fn shared() -> MutexGuard<'static, Client> {
        CLIENT.lock().unwrap()
    }

    // Contexts
    #[cfg(not(feature = "node_interaction"))]
    pub fn create_context(&mut self, config_str: String) -> ApiResult<Arc<ClientContext>> {
        let config: ClientConfig = crate::dispatch::parse_params(&config_str)?;
        let config: InternalClientConfig = config.into();

        let handle = self.next_context_handle;
        self.next_context_handle = handle.wrapping_add(1);
        let context = Arc::new(ClientContext { handle, config });
        self.contexts.insert(handle, context.clone());

        Ok(context)
    }

    #[cfg(feature = "node_interaction")]
    pub fn create_context(&mut self, config_str: String) -> ApiResult<Arc<ClientContext>> {
        let config: ClientConfig = parse_params(&config_str)?;

        let handle = self.next_context_handle;
        self.next_context_handle = handle.wrapping_add(1);

        let context = Arc::new(create_context(handle, config)?);
        self.contexts.insert(handle, context.clone());

        Ok(context)
    }

    pub fn destroy_context(&mut self, handle: ContextHandle) {
        self.contexts.remove(&handle);
    }

    pub fn required_context(&self, context: ContextHandle) -> ApiResult<Arc<ClientContext>> {
        Ok(Arc::clone(
            self.contexts
                .get(&context)
                .ok_or(Error::invalid_context_handle(context))?,
        ))
    }
}
