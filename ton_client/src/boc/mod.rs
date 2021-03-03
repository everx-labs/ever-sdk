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

use serde::{Deserialize, Deserializer};

pub(crate) mod blockchain_config;
pub(crate) mod cache;
mod errors;
pub(crate) mod hash;
pub(crate) mod internal;
pub(crate) mod parse;
pub(crate) mod tvc;

#[cfg(test)]
pub(crate) mod tests;

pub use cache::{
    cache_get, cache_set, cache_unpin,
    BocCacheType, ParamsOfBocCacheGet, ParamsOfBocCacheSet, ParamsOfBocCacheUnpin,
    ResultOfBocCacheGet, ResultOfBocCacheSet,
};
pub use parse::{
    parse_account, parse_block, parse_message, parse_shardstate, parse_transaction, required_boc,
    source_boc, ParamsOfParse, ParamsOfParseShardstate, ResultOfParse,
};
pub use blockchain_config::{
    get_blockchain_config, ParamsOfGetBlockchainConfig, ResultOfGetBlockchainConfig,
};
pub use errors::{Error, ErrorCode};
pub use hash::{get_boc_hash, ParamsOfGetBocHash, ResultOfGetBocHash};
pub use tvc::{get_code_from_tvc, ParamsOfGetCodeFromTvc, ResultOfGetCodeFromTvc};

pub fn default_cache_max_size() -> u32 {
    10 * 1024 // * 1024 = 10 MB
}

fn deserialize_cache_max_size<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_cache_max_size()))
}

#[derive(Deserialize, Debug, Clone, ApiType)]
pub struct BocConfig {
    /// Maximum BOC cache size in kilobytes. Default is 10 MB
    #[serde(
        default = "default_cache_max_size",
        deserialize_with = "deserialize_cache_max_size"
    )]
    pub cache_max_size: u32,
}

impl Default for BocConfig {
    fn default() -> Self {
        Self {
            cache_max_size: default_cache_max_size(),
        }
    }
}

