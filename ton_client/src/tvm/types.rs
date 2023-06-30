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

use super::Error;
use crate::client::{ClientContext, NetworkParams};
use crate::error::ClientResult;
use crate::net::network_params::get_default_params;
use crate::boc::internal::deserialize_object_from_boc;
use std::sync::Arc;
use ton_executor::BlockchainConfig;
use ton_vm::executor::BehaviorModifiers;

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
    /// Overrides standard TVM behaviour.
    /// If set to `true` then CHKSIG always will return `true`.
    pub chksig_always_succeed: Option<bool>,
    /// Signature ID to be used in signature verifying instructions when CapSignatureWithId
    /// capability is enabled
    pub signature_id: Option<i32>,
}

pub(crate) struct ResolvedExecutionOptions {
    pub blockchain_config: Arc<BlockchainConfig>,
    pub signature_id: i32,
    pub block_time: u32,
    pub block_lt: u64,
    pub transaction_lt: u64,
    pub behavior_modifiers: BehaviorModifiers,
}

pub(crate) fn blockchain_config_from_boc(context: &ClientContext, b64: &str) -> ClientResult<BlockchainConfig> {
    let config_params = deserialize_object_from_boc(context, b64, "blockchain config")?;
    BlockchainConfig::with_config(config_params.object)
        .map_err(|err| Error::can_not_read_blockchain_config(err))
}

impl ResolvedExecutionOptions {
    pub async fn from_options(
        context: &Arc<ClientContext>,
        options: Option<ExecutionOptions>,
    ) -> ClientResult<Self> {
        let options = options.unwrap_or_default();

        let params = resolve_network_params(
            context, options.blockchain_config, options.signature_id
        ).await?;

        let block_lt = options
            .block_lt
            .unwrap_or(options.transaction_lt.unwrap_or(1_000_001) - 1);
        let transaction_lt = options.transaction_lt.unwrap_or(block_lt + 1);
        let block_time = options
            .block_time
            .unwrap_or_else(|| (context.env.now_ms() / 1000) as u32);
        let behavior_modifiers = BehaviorModifiers {
            chksig_always_succeed: options.chksig_always_succeed.unwrap_or(false),
            ..Default::default()
        };
        Ok(Self {
            block_lt,
            block_time,
            blockchain_config: params.blockchain_config,
            signature_id: params.global_id,
            transaction_lt,
            behavior_modifiers,
        })
    }
}

pub(crate) async fn resolve_network_params(
    context: &Arc<ClientContext>,
    provided_config: Option<String>,
    provided_global_id: Option<i32>,
) -> ClientResult<NetworkParams> {
    match (provided_config, provided_global_id.or(context.config.network.signature_id)) {
        (Some(config), Some(global_id)) => {
            Ok(NetworkParams {
                blockchain_config: Arc::new(blockchain_config_from_boc(context, &config)?),
                global_id,
            })
        },
        (Some(config), None) => {
            let default = get_default_params(context).await?;
            Ok(NetworkParams {
                blockchain_config: Arc::new(blockchain_config_from_boc(context, &config)?),
                global_id: default.global_id,
            })
        },
        (None, Some(global_id)) => {
            let default = get_default_params(context).await?;
            Ok(NetworkParams {
                blockchain_config: default.blockchain_config,
                global_id,
            })
        },
        (None, None) => {
            get_default_params(context).await
        }
    }
}

