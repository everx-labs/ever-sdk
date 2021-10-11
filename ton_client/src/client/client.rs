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
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};

use super::{AppRequestResult, Error, ParamsOfAppRequest};
use crate::abi::AbiConfig;
use crate::boc::{BocConfig, cache::Bocs};
use crate::crypto::CryptoConfig;
use crate::crypto::boxes::{signing_box::SigningBox, encryption_box::EncryptionBox};
use crate::debot::DEngine;
use crate::error::ClientResult;
use crate::json_interface::interop::ResponseType;
use crate::json_interface::request::Request;
use crate::net::{
    subscriptions::SubscriptionAction, ChainIterator, NetworkConfig, ServerLink,
};

#[cfg(not(feature = "wasm"))]
use super::std_client_env::ClientEnv;
#[cfg(feature = "wasm")]
use super::wasm_client_env::ClientEnv;
use crate::proofs::storage::ProofStorage;
use ton_types::UInt256;

#[derive(Default)]
pub struct Boxes {
    pub(crate) signing_boxes: LockfreeMap<u32, Box<dyn SigningBox + Send + Sync>>,
    pub(crate) encryption_boxes: LockfreeMap<u32, Box<dyn EncryptionBox + Send + Sync>>,
}

#[derive(Debug)]
pub(crate) struct NetworkUID {
    pub(crate) zerostate_root_hash: UInt256,
    pub(crate) first_master_block_root_hash: UInt256,
}

pub struct NetworkContext {
    pub(crate) server_link: Option<ServerLink>,
    pub(crate) subscriptions: Mutex<HashMap<u32, mpsc::Sender<SubscriptionAction>>>,
    pub(crate) iterators: Mutex<HashMap<u32, Arc<Mutex<Box<dyn ChainIterator + Send + Sync>>>>>,
    pub(crate) network_uid: RwLock<Option<Arc<NetworkUID>>>,
}

pub struct ClientContext {
    pub(crate) net: NetworkContext,
    pub(crate) config: ClientConfig,
    pub(crate) env: Arc<ClientEnv>,
    pub(crate) debots: LockfreeMap<u32, Mutex<DEngine>>,
    pub(crate) boxes: Boxes,
    pub(crate) bocs: Bocs,
    pub(crate) blockchain_config: RwLock<Option<Arc<ton_executor::BlockchainConfig>>>,

    pub(crate) app_requests: Mutex<HashMap<u32, oneshot::Sender<AppRequestResult>>>,
    pub(crate) storage: Arc<dyn ProofStorage>,

    next_id: AtomicU32,
}

impl ClientContext {
    pub(crate) fn get_server_link(&self) -> ClientResult<&ServerLink> {
        self.net
            .server_link
            .as_ref()
            .ok_or_else(|| Error::net_module_not_init())
    }

    pub async fn set_timer(&self, ms: u64) -> ClientResult<()> {
        self.env.set_timer(ms).await
    }

    pub fn new(config: ClientConfig) -> ClientResult<ClientContext> {
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

        let bocs = Bocs::new(config.boc.cache_max_size);
        let storage: Arc<dyn ProofStorage> = if config.cache_proofs {
            Arc::new(crate::proofs::storage::LocalStorage::new(config.local_storage_path.clone()))
        } else {
            Arc::new(crate::proofs::storage::InMemoryProofStorage::new())
        };
        Ok(Self {
            net: NetworkContext {
                server_link,
                subscriptions: Default::default(),
                iterators: Default::default(),
                network_uid: Default::default(),
            },
            config,
            env,
            debots: LockfreeMap::new(),
            boxes: Default::default(),
            bocs,
            blockchain_config: RwLock::new(None),
            app_requests: Mutex::new(HashMap::new()),
            storage,
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

#[derive(Deserialize, Debug, Clone, ApiType)]
pub struct ClientConfig {
    #[serde(default, deserialize_with = "deserialize_network_config")]
    pub network: NetworkConfig,
    #[serde(default, deserialize_with = "deserialize_crypto_config")]
    pub crypto: CryptoConfig,
    #[serde(default, deserialize_with = "deserialize_abi_config")]
    pub abi: AbiConfig,
    #[serde(default, deserialize_with = "deserialize_boc_config")]
    pub boc: BocConfig,

    /// For file based storage is a folder name where SDK will store its data.
    /// For browser based is a browser async storage key prefix.
    /// Default (recommended) value is "~/.tonclient" for native environments and ".tonclient"
    /// for web-browser.
    pub local_storage_path: Option<String>,

    /// Cache proofs in the local storage. Default is `true`.
    #[serde(
        default = "default_cache_proofs",
        deserialize_with = "deserialize_cache_proofs"
    )]
    pub cache_proofs: bool,
}

fn default_cache_proofs() -> bool {
    true
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

fn deserialize_cache_proofs<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<bool, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_cache_proofs()))
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            network: Default::default(),
            crypto: Default::default(),
            abi: Default::default(),
            boc: Default::default(),
            local_storage_path: Default::default(),
            cache_proofs: default_cache_proofs(),
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
