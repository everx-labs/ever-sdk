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
use std::sync::Arc;
use ton_block::{Deserializable, GlobalCapabilities};
use ton_executor::BlockchainConfig;

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

pub(crate) fn mainnet_config() -> (BlockchainConfig, i32) {
    let bytes = include_bytes!("../mainnet_config_10660619.boc");
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

    let (config, global_id) = if let Ok(link) = context.get_server_link() {
        query_network_params(link)
            .await
            .unwrap_or_else(|_| mainnet_config())
    } else {
        mainnet_config()
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
