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
 *
 */

use super::Error;
use crate::boc::internal::deserialize_object_from_boc;
use crate::client::ClientContext;
use crate::error::ClientResult;
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
    pub blockchain_config: BlockchainConfig,
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
        context: &std::sync::Arc<ClientContext>,
        options: Option<ExecutionOptions>,
    ) -> ClientResult<Self> {
        let options = options.unwrap_or_default();

        let config = if let Some(config) = options.blockchain_config {
            blockchain_config_from_boc(context, &config).await?
        } else {
            Default::default()
        };

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
