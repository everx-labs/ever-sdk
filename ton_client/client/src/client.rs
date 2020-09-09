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
use crate::error::{ApiResult, ApiError};
use super::{JsonResponse, InteropContext, OnResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use ton_sdk::{NetworkConfig, AbiConfig};

#[cfg(feature = "node_interaction")]
use ton_sdk::NodeClient;

#[cfg(feature = "node_interaction")]
use tokio::runtime::Runtime;
use crate::get_api;

lazy_static! {
    static ref HANDLERS: DispatchTable = create_handlers();
    static ref CLIENT: Mutex<Client> = Mutex::new(Client::new());
}

#[derive(Serialize, Deserialize, TypeInfo, Clone)]
pub struct ResultOfVersion {
    /// core version
    pub version: String,
}

fn create_handlers() -> DispatchTable {
    let mut handlers = DispatchTable::new();
    crate::crypto::register(&mut handlers);
    crate::contracts::register(&mut handlers);
    crate::abi::register(&mut handlers);
    crate::tvm::register(&mut handlers);

    //TODO: uncomment this when cell module will be ready
    // crate::cell::register(&mut handlers);

    #[cfg(feature = "node_interaction")]
    crate::queries::register(&mut handlers);

    handlers.call_no_args(
        "config.get_api_reference",
        |_context| Ok(get_api()),
    );
    handlers.call_no_args(
        "version", 
        |_| Ok(ResultOfVersion { version: env!("CARGO_PKG_VERSION").to_owned() }));
    handlers
}

fn sync_request(context: std::sync::Arc<ClientContext>, method: String, params_json: String) -> JsonResponse {
    HANDLERS.sync_dispatch(context, method, params_json)
}

fn async_request(
    context: std::sync::Arc<ClientContext>,
    method: String,
    params_json: String,
    request_id: u32,
    on_result: OnResult
) {
    HANDLERS.async_dispatch(context, method, params_json, request_id, on_result)
}

pub struct ClientContext {
    #[cfg(feature = "node_interaction")]
    pub client: Option<NodeClient>,
    #[cfg(feature = "node_interaction")]
    pub runtime: Runtime,
    pub handle: InteropContext,
    pub config: InternalClientConfig
}

#[cfg(feature = "node_interaction")]
impl ClientContext {
    pub fn get_client(&self) -> ApiResult<&NodeClient> {
        self.client.as_ref().ok_or(ApiError::sdk_not_init())
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

#[derive(Deserialize, Debug, Clone)]
pub struct ClientConfig {
    pub network: Option<NetworkConfig>,
    pub crypto: Option<CryptoConfig>,
    pub abi: Option<AbiConfig>
}

#[derive(Debug, Clone)]
pub struct InternalClientConfig {
    pub network: Option<NetworkConfig>,
    pub crypto: CryptoConfig,
    pub abi: AbiConfig
}

impl From<ClientConfig> for InternalClientConfig {
    fn from(config: ClientConfig) -> Self {
        InternalClientConfig {
            network: config.network,
            crypto: config.crypto.unwrap_or_default(),
            abi: config.abi.unwrap_or_default()
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResultOfCreateContext {
    pub handle: InteropContext,
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

        #[cfg(not(feature = "node_interaction"))]
            self.contexts.insert(handle, Arc::new(ClientContext {
            handle,
            config,
        }));

        Ok(ResultOfCreateContext {
            handle
        })
    }

    #[cfg(feature = "node_interaction")]
    fn create_context_internal(&mut self, config_str: String) -> ApiResult<ResultOfCreateContext> {
        let config: ClientConfig = crate::dispatch::parse_params(&config_str)?;
        let config: InternalClientConfig = config.into();

        let client = if let Some(net_config) = &config.network {
            if net_config.out_of_sync_threshold() > config.abi.message_expiration_timeout() as i64 / 2 {
                return Err(ApiError::invalid_params(
                    &config_str,
                    format!(
r#"`out_of_sync_threshold` can not be more then `message_expiration_timeout / 2`.
`out_of_sync_threshold` = {}, `message_expiration_timeout` = {}
Note that default values are used if parameters are omitted in config"#,
                        net_config.out_of_sync_threshold(), config.abi.message_expiration_timeout())
                    ));
            }
            Some(NodeClient::new(net_config.clone()))
        } else {
            None
        };

        let runtime = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|err| ApiError::cannot_create_runtime(err))?;

        let handle = self.next_context_handle;
        self.next_context_handle = handle.wrapping_add(1);

        self.contexts.insert(handle, Arc::new(ClientContext {
            handle,
            client,
            runtime,
            config,
        }));

        Ok(ResultOfCreateContext {
            handle
        })
    }

    pub fn create_context(&mut self, config: String) -> JsonResponse {
        match self.create_context_internal(config) {
            Ok(result) => JsonResponse::from_result(serde_json::to_string(&result).unwrap()),
            Err(err) => JsonResponse::from_error(err)
        }
    }

    pub fn destroy_context(&mut self, handle: InteropContext) {
        self.contexts.remove(&handle);
    }

    pub fn required_context(&self, context: InteropContext) -> ApiResult<Arc<ClientContext>> {
        Ok(Arc::clone(
            self.contexts.get(&context)
                .ok_or(ApiError::invalid_context_handle(context))?
        ))
    }

    pub fn json_sync_request(handle: InteropContext, method_name: String, params_json: String) -> JsonResponse {
        let context = Self::shared().required_context(handle);
        match context {
            Ok(context) => sync_request(context, method_name, params_json),
            Err(err) => JsonResponse::from_error(err)
        }
    }
        
    pub fn json_async_request(
        handle: InteropContext,
        method_name: String,
        params_json: String,
        request_id: u32,
        on_result: OnResult
    ) {
        let context = Self::shared().required_context(handle);
        match context {
            Ok(context) => {
                async_request(context, method_name, params_json, request_id, on_result);
            }
            Err(err) => {
                JsonResponse::from_error(err).send(on_result, request_id, 1);
            }
        }
    }

    pub fn get_api(&self) -> api_doc::api::API {
        HANDLERS.get_api()
    }
}


