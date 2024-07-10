/*
* Copyright 2018-2021 EverX Labs Ltd.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific EVERX DEV software governing permissions and
* limitations under the License.
*/

use serde::{Deserialize, Deserializer};

pub(crate) mod blockchain_config;
pub(crate) mod cache;
pub(crate) mod common;
pub(crate) mod encode;
mod errors;
pub mod internal;
pub(crate) mod parse;
pub(crate) mod state_init;

pub(crate) mod encode_external_in_message;
#[cfg(test)]
pub(crate) mod tests;
pub(crate) mod tvc;

pub use blockchain_config::{
    get_blockchain_config, ParamsOfGetBlockchainConfig, ResultOfGetBlockchainConfig,
};
pub use cache::{
    cache_get, cache_set, cache_unpin, BocCacheType, CachedBoc, ParamsOfBocCacheGet,
    ParamsOfBocCacheSet, ParamsOfBocCacheUnpin, ResultOfBocCacheGet, ResultOfBocCacheSet,
};
pub use common::{
    get_boc_depth, get_boc_hash, ParamsOfGetBocDepth, ParamsOfGetBocHash, ResultOfGetBocDepth,
    ResultOfGetBocHash,
};
pub use encode::{encode_boc, BuilderOp, ParamsOfEncodeBoc, ResultOfEncodeBoc};
pub use encode_external_in_message::{
    encode_external_in_message, ParamsOfEncodeExternalInMessage, ResultOfEncodeExternalInMessage,
};
pub use errors::{Error, ErrorCode};
pub use parse::{
    parse_account, parse_block, parse_message, parse_shardstate, parse_transaction, required_boc,
    source_boc, ParamsOfParse, ParamsOfParseShardstate, ResultOfParse,
};
pub use state_init::{
    decode_state_init, encode_state_init, get_code_from_tvc, get_code_salt, get_compiler_version,
    get_compiler_version_from_cell, set_code_salt, ParamsOfDecodeStateInit,
    ParamsOfEncodeStateInit, ParamsOfGetCodeFromTvc, ParamsOfGetCodeSalt,
    ParamsOfGetCompilerVersion, ParamsOfSetCodeSalt, ResultOfDecodeStateInit,
    ResultOfEncodeStateInit, ResultOfGetCodeFromTvc, ResultOfGetCodeSalt,
    ResultOfGetCompilerVersion, ResultOfSetCodeSalt,
};

pub use tvc::{decode_tvc, Tvc, TvcV1};

pub fn default_cache_max_size() -> u32 {
    10 * 1024 // * 1024 = 10 MB
}

fn deserialize_cache_max_size<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_cache_max_size()))
}

#[derive(Deserialize, Serialize, Debug, Clone, ApiType)]
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
