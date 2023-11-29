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
*
*/

use crate::client::{ClientContext, NetworkParams};
use crate::error::ClientResult;
use crate::{
    boc::{
        blockchain_config::{extract_config_from_block, extract_config_from_zerostate},
        internal::deserialize_object_from_base64,
    },
    net::{OrderBy, ParamsOfQueryCollection, ServerLink, SortDirection},
};
use serde_json::Value;
use std::{fs, path::Path, sync::Arc};
use ton_block::{Deserializable, GlobalCapabilities};
use ton_executor::BlockchainConfig;

const ACKI_CONFIG_FILE: &str = "ACKI_CONFIG_FILE";
const ACKI_GLOBAL_ID: &str = "ACKI_GLOBAL_ID";
const DEFAULT_ACKI_CONFIG_FILE: &str = "src/blockchain.conf.json";
const DEFAULT_ACKI_GLOBAL_ID: i32 = 100;

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ResultOfGetSignatureId {
    /// Signature ID for configured network if it should be used in messages signature
    pub signature_id: Option<i32>,
}

/// Returns signature ID for configured network if it should be used in messages signature
#[api_function]
pub async fn get_signature_id(context: std::sync::Arc<ClientContext>) -> ClientResult<ResultOfGetSignatureId> {
    let params = get_default_params(&context).await?;
    if params.blockchain_config.has_capability(GlobalCapabilities::CapSignatureWithId) {
        Ok(ResultOfGetSignatureId { signature_id: Some(params.global_id) })
    } else {
        Ok(ResultOfGetSignatureId { signature_id: None })
    }
}

pub(crate) fn offline_config() -> (BlockchainConfig, i32) {
    let bytes = include_bytes!("../default_config.boc");
    (
        BlockchainConfig::with_config(
            ton_block::ConfigParams::construct_from_bytes(bytes).unwrap()
        ).unwrap(),
        42
    )
}

pub(crate) async fn get_default_params(context: &Arc<ClientContext>) -> ClientResult<NetworkParams> {
    if let Some(params) = &*context.network_params.read().await {
        return Ok(params.clone());
    }

    let mut params_lock = context.network_params.write().await;
    if let Some(params) = &*params_lock {
        return Ok(params.clone());
    }

    let (config, global_id) = if let Ok(_) = context.get_server_link() {
        // query_network_params(link).await?
        query_network_params_from_file().await?
    } else {
        offline_config()
    };
    let params = NetworkParams { 
        blockchain_config: Arc::new(config),
        global_id
     };

    *params_lock = Some(params.clone());

    Ok(params)
}

pub(crate) async fn query_network_params(link: &ServerLink) -> ClientResult<(BlockchainConfig, i32)> {
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

    let (config, global_id) = if let Some(block_boc) = key_block[0]["boc"].as_str() {
        let block = deserialize_object_from_base64(block_boc, "block")?;
        (extract_config_from_block(&block.object)?, block.object.global_id())
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
            crate::tvm::Error::can_not_read_blockchain_config("Can not find key block or zerostate"))?;

        let zerostate = deserialize_object_from_base64(boc, "block")?;
        (extract_config_from_zerostate(&zerostate.object)?, zerostate.object.global_id())
    };

    let config = BlockchainConfig::with_config(config)
        .map_err(|err| crate::tvm::Error::can_not_read_blockchain_config(err))?;
    Ok((config, global_id))
}

pub(crate) async fn query_network_params_from_file() -> ClientResult<(BlockchainConfig, i32)> {
    let config_file_name = std::env::var(ACKI_CONFIG_FILE)
        .ok()
        .unwrap_or(DEFAULT_ACKI_CONFIG_FILE.to_owned());

    let global_id = std::env::var(ACKI_GLOBAL_ID)
        .ok()
        .and_then(|num| i32::from_str_radix(&num, 10).ok())
        .unwrap_or(DEFAULT_ACKI_GLOBAL_ID);

    let blockchain_config_json = read_str(&config_file_name)?;
    let config = blockchain_config_from_json(&blockchain_config_json)?;
    Ok((config, global_id))
}

fn read_str(path: &str) -> ClientResult<String> {
    Ok(fs::read_to_string(Path::new(path))
        .map_err(|err| crate::tvm::Error::can_not_read_blockchain_config_from_file(err))?)
}

fn blockchain_config_from_json(json: &str) -> ClientResult<BlockchainConfig> {
    let map =
        serde_json::from_str::<serde_json::Map<String, Value>>(&json)
        .map_err(|err| crate::tvm::Error::json_deserialization_failed(err))?;
    let config_params =
        ton_block_json::parse_config(&map)
        .map_err(|err| crate::tvm::Error::can_not_parse_config(err))?;
    BlockchainConfig::with_config(config_params)
        .map_err(|err| crate::tvm::Error::can_not_convert_config(err))
}
