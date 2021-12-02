/*
 * Copyright 2018-2021 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 *
 */

use super::Error;
use crate::{boc::{
    blockchain_config::{extract_config_from_block, extract_config_from_zerostate},
    internal::{deserialize_object_from_base64, deserialize_object_from_boc},
}, net::ServerLink};
use crate::net::ParamsOfQueryCollection;
use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::net::{OrderBy, SortDirection};
use std::sync::Arc;
use ton_block::Deserializable;
use ton_executor::BlockchainConfig;

#[derive(Serialize, Deserialize, ApiType, Clone, Default)]
pub struct ExecutionOptions {
    /// boc with config
    pub blockchain_config: Option<String>,
    /// time that is used as transaction time
    pub block_time: Option<u32>,
    /// block logical time
    pub block_lt: Option<u64>,
    /// transaction logical time
    pub transaction_lt: Option<u64>,
}

pub(crate) struct ResolvedExecutionOptions {
    pub blockchain_config: Arc<BlockchainConfig>,
    pub block_time: u32,
    pub block_lt: u64,
    pub transaction_lt: u64,
}

pub(crate) async fn blockchain_config_from_boc(context: &ClientContext, b64: &str) -> ClientResult<BlockchainConfig> {
    let config_params = deserialize_object_from_boc(context, b64, "blockchain config").await?;
    BlockchainConfig::with_config(config_params.object)
        .map_err(|err| Error::can_not_read_blockchain_config(err))
}

impl ResolvedExecutionOptions {
    pub async fn from_options(
        context: &Arc<ClientContext>,
        options: Option<ExecutionOptions>,
    ) -> ClientResult<Self> {
        let options = options.unwrap_or_default();

        let config = resolve_blockchain_config(context,options.blockchain_config).await?;

        let block_lt = options
            .block_lt
            .unwrap_or(options.transaction_lt.unwrap_or(1_000_001) - 1);
        let transaction_lt = options.transaction_lt.unwrap_or(block_lt + 1);
        let block_time = options
            .block_time
            .unwrap_or_else(|| (context.env.now_ms() / 1000) as u32);

        Ok(Self {
            block_lt,
            block_time,
            blockchain_config: config,
            transaction_lt,
        })
    }
}

pub async fn resolve_blockchain_config(
    context: &Arc<ClientContext>,
    provided_config: Option<String>,
) -> ClientResult<Arc<BlockchainConfig>> {
    if let Some(config) = provided_config {
        blockchain_config_from_boc(context, &config).await.map(Arc::new)
    } else {
        get_default_config(context).await
    }
}

pub(crate) fn mainnet_config() -> BlockchainConfig {
    let bytes = include_bytes!("../mainnet_config_10660619.boc");
    BlockchainConfig::with_config(
        ton_block::ConfigParams::construct_from_bytes(bytes).unwrap()
    ).unwrap()
}

pub(crate) async fn get_default_config(context: &Arc<ClientContext>) -> ClientResult<Arc<BlockchainConfig>> {
    if let Some(config) = &*context.blockchain_config.read().await {
        return Ok(config.clone());
    }

    let mut config_lock = context.blockchain_config.write().await;
    if let Some(config) = &*config_lock {
        return Ok(config.clone());
    }

    let config = if let Ok(link) = context.get_server_link() {
        get_network_config(link)
            .await
            .unwrap_or_else(|_| mainnet_config())
    } else {
        mainnet_config()
    };
    let config = Arc::new(config);

    *config_lock = Some(config.clone());

    Ok(config)
}

pub(crate) async fn get_network_config(link: &ServerLink) -> ClientResult<BlockchainConfig> {
    let key_block = link.query_collection(ParamsOfQueryCollection {
        collection: "blocks".to_owned(),
        filter: Some(serde_json::json!({
            "key_block": { "eq": true },
            "workchain_id": { "eq": -1 },
        })),
        order: Some(vec![OrderBy { path: "seq_no".to_owned(), direction: SortDirection::DESC }]),
        limit: Some(1),
        result: "boc".to_owned(),
    }, None).await?;

    let config = if let Some(block_boc) = key_block[0]["boc"].as_str() {
        let block = deserialize_object_from_base64(block_boc, "block")?;
        extract_config_from_block(block.object)?
    } else {
        let zerostate = link.query_collection(ParamsOfQueryCollection {
            collection: "zerostates".to_owned(),
            filter: Some(serde_json::json!({
                "id": { "eq": "zerostate:-1" },
            })),
            result: "boc".to_owned(),
            ..Default::default()
        }, None).await?;

        let boc = zerostate[0]["boc"].as_str().ok_or(
            Error::can_not_read_blockchain_config("Can not find key block or zerostate"))?;

        let zerostate = deserialize_object_from_base64(boc, "block")?;
        extract_config_from_zerostate(zerostate.object)?
    };

    BlockchainConfig::with_config(config)
        .map_err(|err| Error::can_not_read_blockchain_config(err))
}
