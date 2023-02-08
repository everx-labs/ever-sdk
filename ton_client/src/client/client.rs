/*
* Copyright 2018-2021 TON Labs LTD.
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
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex, RwLock};
use ton_client_processing::MessageMonitor;

#[cfg(not(feature = "wasm-base"))]
use super::std_client_env::ClientEnv;
#[cfg(feature = "wasm-base")]
use super::wasm_client_env::ClientEnv;
use super::{AppRequestResult, Error, ParamsOfAppRequest};
use crate::abi::AbiConfig;
use crate::boc::{cache::Bocs, BocConfig};
use crate::client::storage::KeyValueStorage;
use crate::client::{BindingConfig, update_binding_config};
use crate::crypto::boxes::crypto_box::{CryptoBox, DerivedKeys};
use crate::crypto::boxes::{encryption_box::EncryptionBox, signing_box::SigningBox};
use crate::crypto::CryptoConfig;
use crate::debot::DEngine;
use crate::error::ClientResult;
use crate::json_interface::interop::ResponseType;
use crate::json_interface::request::Request;
use crate::net::{NetworkConfig, NetworkContext, ServerLink};
use crate::processing::SdkServices;
use crate::proofs::ProofsConfig;

#[derive(Default)]
pub struct Boxes {
    pub(crate) crypto_boxes: LockfreeMap<u32, CryptoBox>,
    pub(crate) signing_boxes: LockfreeMap<u32, Box<dyn SigningBox>>,
    pub(crate) encryption_boxes: LockfreeMap<u32, Box<dyn EncryptionBox>>,
}

pub struct ClientContext {
    next_id: AtomicU32,

    // context
    pub(crate) config: ClientConfig,
    pub(crate) blockchain_config: RwLock<Option<Arc<ton_executor::BlockchainConfig>>>,
    pub(crate) app_requests: Mutex<HashMap<u32, oneshot::Sender<AppRequestResult>>>,

    // client module
    pub(crate) env: Arc<ClientEnv>,

    // crypto module
    pub(crate) boxes: Boxes,
    pub(crate) derived_keys: DerivedKeys,

    // boc module
    pub(crate) bocs: Arc<Bocs>,

    // net module
    pub(crate) net: Arc<NetworkContext>,

    // processing module
    pub(crate) message_monitor: Arc<MessageMonitor<SdkServices>>,

    // proofs module
    pub(crate) proofs_storage: RwLock<Option<Arc<dyn KeyValueStorage>>>,

    // debot module
    pub(crate) debots: LockfreeMap<u32, Mutex<DEngine>>,
}

impl ClientContext {
    pub(crate) fn get_server_link(&self) -> ClientResult<&ServerLink> {
        self.net.get_server_link()
    }

    pub async fn set_timer(&self, ms: u64) -> ClientResult<()> {
        self.env.set_timer(ms).await
    }

    pub fn now_ms(&self) -> u64 {
        self.env.now_ms()
    }

    pub fn new(config: ClientConfig) -> ClientResult<ClientContext> {
        update_binding_config(&config.binding);
        let env = Arc::new(ClientEnv::new()?);

        let server_link = if config.network.server_address.is_some()
            || config.network.endpoints.is_some()
        {
            if config.network.out_of_sync_threshold > config.abi.message_expiration_timeout / 2 {
                return Err(Error::invalid_config(format!(
                    r#"`out_of_sync_threshold` can not be more then `message_expiration_timeout / 2`.
`out_of_sync_threshold` = {}, `message_expiration_timeout` = {}
Note that default values are used if parameters are omitted in config"#,
                    config.network.out_of_sync_threshold, config.abi.message_expiration_timeout
                )));
            }
            Some(ServerLink::new(config.network.clone(), env.clone())?)
        } else {
            None
        };

        let bocs = Arc::new(Bocs::new(config.boc.cache_max_size));
        let net = Arc::new(NetworkContext {
            env: env.clone(),
            server_link,
            subscriptions: Default::default(),
            iterators: Default::default(),
            network_uid: Default::default(),
        });
        let message_monitor = Arc::new(MessageMonitor::new(SdkServices::new(
            net.clone(),
            bocs.clone(),
        )));
        Ok(Self {
            net,
            message_monitor,
            config,
            env: env.clone(),
            debots: LockfreeMap::new(),
            boxes: Default::default(),
            bocs,
            blockchain_config: RwLock::new(None),
            app_requests: Mutex::new(HashMap::new()),
            proofs_storage: Default::default(),
            derived_keys: DerivedKeys::new(env),
            next_id: AtomicU32::new(1),
        })
    }

    pub(crate) fn get_next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) async fn app_request<R: DeserializeOwned>(
        &self,
        callback: &Request,
        params: impl Serialize,
    ) -> ClientResult<R> {
        let id = self.get_next_id();
        let (sender, receiver) = oneshot::channel();
        self.app_requests.lock().await.insert(id, sender);

        let params = serde_json::to_value(params).map_err(Error::cannot_serialize_result)?;

        callback.response(
            ParamsOfAppRequest {
                app_request_id: id,
                request_data: params,
            },
            ResponseType::AppRequest as u32,
        );
        let result = receiver
            .await
            .map_err(|err| Error::can_not_receive_request_result(err))?;

        match result {
            AppRequestResult::Error { text } => Err(Error::app_request_error(&text)),
            AppRequestResult::Ok { result } => serde_json::from_value(result)
                .map_err(|err| Error::can_not_parse_request_result(err)),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, ApiType)]
pub struct ClientConfig {
    #[serde(default, deserialize_with = "deserialize_binding_config")]
    pub binding: BindingConfig,
    #[serde(default, deserialize_with = "deserialize_network_config")]
    pub network: NetworkConfig,
    #[serde(default, deserialize_with = "deserialize_crypto_config")]
    pub crypto: CryptoConfig,
    #[serde(default, deserialize_with = "deserialize_abi_config")]
    pub abi: AbiConfig,
    #[serde(default, deserialize_with = "deserialize_boc_config")]
    pub boc: BocConfig,
    #[serde(default, deserialize_with = "deserialize_proofs_config")]
    pub proofs: ProofsConfig,

    /// For file based storage is a folder name where SDK will store its data.
    /// For browser based is a browser async storage key prefix.
    /// Default (recommended) value is "~/.tonclient" for native environments and ".tonclient"
    /// for web-browser.
    pub local_storage_path: Option<String>,
}

fn deserialize_binding_config<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BindingConfig, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(Default::default()))
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

fn deserialize_boc_config<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BocConfig, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(Default::default()))
}

fn deserialize_proofs_config<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<ProofsConfig, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(Default::default()))
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            binding: Default::default(),
            network: Default::default(),
            crypto: Default::default(),
            abi: Default::default(),
            boc: Default::default(),
            proofs: Default::default(),
            local_storage_path: Default::default(),
        }
    }
}

pub(crate) struct AppObject<P: Serialize, R: DeserializeOwned> {
    context: Arc<ClientContext>,
    object_handler: Arc<Request>,
    phantom: std::marker::PhantomData<(P, R)>,
}

impl<P, R> AppObject<P, R>
where
    P: Serialize,
    R: DeserializeOwned,
{
    pub fn new(context: Arc<ClientContext>, object_handler: Arc<Request>) -> AppObject<P, R> {
        AppObject {
            context,
            object_handler,
            phantom: std::marker::PhantomData,
        }
    }

    pub async fn call(&self, params: P) -> ClientResult<R> {
        self.context.app_request(&self.object_handler, params).await
    }

    pub fn notify(&self, params: P) {
        self.object_handler
            .response(params, ResponseType::AppNotify as u32)
    }
}
