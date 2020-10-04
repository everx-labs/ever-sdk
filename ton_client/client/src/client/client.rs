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

use crate::dispatch::DispatchTable;
use crate::error::ApiResult;
use crate::{InteropContext, JsonResponse};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use ton_sdk::AbiConfig;

use crate::net::{NetModule, NetworkConfig, NodeClient};

use super::std_client_env::StdClientEnv;
use super::{ClientEnv, Error};
use crate::abi::AbiModule;
use crate::boc::BocModule;
use crate::client::{register_callback, ClientModule};
use crate::crypto::CryptoModule;
use crate::processing::ProcessingModule;

lazy_static! {
    static ref HANDLERS: DispatchTable = create_handlers();
    static ref CLIENT: Mutex<Client> = Mutex::new(Client::new());
}

pub(crate) fn get_handlers() -> &'static DispatchTable {
    return &HANDLERS;
}

pub type Callback = dyn Fn(u32, &str, &str, u32) + Send + Sync;

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ResultOfVersion {
    /// core version
    pub version: String,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfUnregisterCallback {
    /// Registered callback ID
    pub callback_id: u32,
}

fn create_handlers() -> DispatchTable {
    let mut handlers = DispatchTable::new();
    handlers.call_raw_async("client.register_callback", register_callback);
    crate::tvm::register(&mut handlers);

    handlers.register::<ClientModule>();
    handlers.register::<CryptoModule>();
    handlers.register::<AbiModule>();
    handlers.register::<BocModule>();
    handlers.register::<ProcessingModule>();

    #[cfg(feature = "node_interaction")]
    handlers.register::<NetModule>();

    handlers
}

fn sync_request(
    context: std::sync::Arc<ClientContext>,
    function: String,
    params_json: String,
) -> JsonResponse {
    HANDLERS.sync_dispatch(context, function, params_json)
}

fn async_request(
    context: std::sync::Arc<ClientContext>,
    function: String,
    params_json: String,
    request_id: u32,
    on_result: Box<Callback>,
) {
    HANDLERS.async_dispatch(context, function, params_json, request_id, on_result)
}

pub struct ClientContext {
    #[cfg(feature = "node_interaction")]
    pub(crate) client: Option<NodeClient>,
    #[cfg(feature = "node_interaction")]
    pub(crate) sdk_client: Option<ton_sdk::NodeClient>,
    #[cfg(feature = "node_interaction")]
    _async_runtime: Option<tokio::runtime::Runtime>,
    #[cfg(feature = "node_interaction")]
    pub(crate) async_runtime_handle: tokio::runtime::Handle,
    pub(crate) config: InternalClientConfig,
    pub(crate) callbacks: lockfree::map::Map<u32, std::sync::Arc<Callback>>,
    pub(crate) env: Arc<dyn ClientEnv + Send + Sync>,
}

#[cfg(feature = "node_interaction")]
impl ClientContext {
    pub(crate) fn get_client(&self) -> ApiResult<&NodeClient> {
        self.client.as_ref().ok_or(Error::net_module_not_init())
    }

    pub(crate) fn get_sdk_client(&self) -> ApiResult<&ton_sdk::NodeClient> {
        self.sdk_client.as_ref().ok_or(Error::net_module_not_init())
    }

    pub(crate) fn get_callback(&self, callback_id: u32) -> ApiResult<std::sync::Arc<Callback>> {
        Ok(self
            .callbacks
            .get(&callback_id)
            .ok_or(Error::callback_not_registered(callback_id))?
            .val()
            .clone())
    }

    pub(crate) fn send_callback_result<S: serde::Serialize>(
        &self,
        callback_id: u32,
        result: S,
    ) -> ApiResult<()> {
        let callback = self.get_callback(callback_id)?;
        let response = JsonResponse::from_result(
            serde_json::to_string(&result)
                .map_err(|e| Error::callback_params_cant_be_converted_to_json(e))?,
        );
        response.send(&*callback, callback_id.clone(), 0);
        Ok(())
    }
}

pub struct Client {
    next_context_handle: InteropContext,
    contexts: HashMap<InteropContext, Arc<ClientContext>>,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct ResultOfCreateContext {
    pub handle: InteropContext,
}

#[cfg(feature = "node_interaction")]
pub fn create_context(config: ClientConfig) -> ApiResult<ClientContext> {
    let config: InternalClientConfig = config.into();

    let std_env = Arc::new(StdClientEnv::new()?);

    let (client, sdk_client) = if let Some(net_config) = &config.network {
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
        client,
        sdk_client,
        _async_runtime: async_runtime,
        async_runtime_handle,
        config,
        callbacks: Default::default(),
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
    fn create_context_internal(&mut self, config_str: String) -> ApiResult<ResultOfCreateContext> {
        let config: ClientConfig = crate::dispatch::parse_params(&config_str)?;
        let config: InternalClientConfig = config.into();

        let handle = self.next_context_handle;
        self.next_context_handle = handle.wrapping_add(1);

        self.contexts
            .insert(handle, Arc::new(ClientContext { handle, config }));

        Ok(ResultOfCreateContext { handle })
    }

    #[cfg(feature = "node_interaction")]
    fn create_context_internal(&mut self, config_str: String) -> ApiResult<ResultOfCreateContext> {
        let config: ClientConfig = crate::dispatch::parse_params(&config_str)?;

        let handle = self.next_context_handle;
        self.next_context_handle = handle.wrapping_add(1);

        self.contexts
            .insert(handle, Arc::new(create_context(config)?));

        Ok(ResultOfCreateContext { handle })
    }

    pub fn create_context(&mut self, config: String) -> JsonResponse {
        match self.create_context_internal(config) {
            Ok(result) => JsonResponse::from_result(serde_json::to_string(&result).unwrap()),
            Err(err) => JsonResponse::from_error(err),
        }
    }

    pub fn destroy_context(&mut self, handle: InteropContext) {
        self.contexts.remove(&handle);
    }

    pub fn required_context(&self, context: InteropContext) -> ApiResult<Arc<ClientContext>> {
        Ok(Arc::clone(
            self.contexts
                .get(&context)
                .ok_or(Error::invalid_context_handle(context))?,
        ))
    }

    pub fn json_sync_request(
        handle: InteropContext,
        function: String,
        params_json: String,
    ) -> JsonResponse {
        let context = Self::shared().required_context(handle);
        match context {
            Ok(context) => sync_request(context, function, params_json),
            Err(err) => JsonResponse::from_error(err),
        }
    }

    pub fn json_async_request(
        handle: InteropContext,
        function: String,
        params_json: String,
        request_id: u32,
        on_result: Box<Callback>,
    ) {
        let context = Self::shared().required_context(handle);
        match context {
            Ok(context) => {
                async_request(context, function, params_json, request_id, on_result);
            }
            Err(err) => {
                JsonResponse::from_error(err).send(&*on_result, request_id, 1);
            }
        }
    }
}
