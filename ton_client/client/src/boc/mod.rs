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

mod blockchain_config;
mod errors;
pub(crate) mod internal;
mod parse;
// TODO: uncomment when module will be ready
//mod cell;

pub(crate) mod build;
#[cfg(test)]
mod tests;

pub use crate::boc::blockchain_config::{
    get_blockchain_config, ParamsOfGetBlockchainConfig, ResultOfGetBlockchainConfig,
};
pub use crate::boc::parse::{
    parse_account, parse_block, parse_message, parse_transaction, ParamsOfParse, ResultOfParse,
};
pub use build::{build_account, ParamsOfBuildAccount, ResultOfBuildAccount};
pub use errors::{Error, ErrorCode};

use crate::dispatch::{ModuleReg, Registrar};

/// BOC manipulation module.
#[derive(ApiModule)]
#[api_module(name = "boc")]
pub(crate) struct BocModule;
impl ModuleReg for BocModule {
    fn reg(reg: &mut Registrar) {
        reg.f(parse_message, parse::parse_message_api);
        reg.f(parse_transaction, parse::parse_transaction_api);
        reg.f(parse_account, parse::parse_account_api);
        reg.f(parse_block, parse::parse_block_api);
        reg.f(
            get_blockchain_config,
            blockchain_config::get_blockchain_config_api,
        );
    }
}
