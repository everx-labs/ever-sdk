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

pub(crate) mod blockchain_config;
mod errors;
pub(crate) mod hash;
pub(crate) mod internal;
pub(crate) mod parse;

#[cfg(test)]
mod tests;

pub use crate::boc::parse::{
    parse_account, parse_block, parse_message, parse_shardstate, parse_transaction, required_boc,
    source_boc, ParamsOfParse, ParamsOfParseShardstate, ResultOfParse,
};
pub use blockchain_config::{
    get_blockchain_config, ParamsOfGetBlockchainConfig, ResultOfGetBlockchainConfig,
};
pub use errors::{Error, ErrorCode};
pub use hash::{get_boc_hash, ParamsOfGetBocHash, ResultOfGetBocHash};
