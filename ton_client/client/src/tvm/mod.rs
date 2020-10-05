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

mod check_transaction;
mod errors;
mod execute_get;
mod execute_message;
mod execute_message_full;
mod execute_message_tvm_only;
mod types;

#[cfg(test)]
mod tests;

pub use errors::{Error, ErrorCode};
pub use execute_get::{execute_get, ParamsOfExecuteGet, ResultOfExecuteGet};
pub use execute_message::{execute_message, ParamsOfExecuteMessage, ResultOfExecuteMessage};
pub use types::ExitCode;
pub(crate) use types::StackJson;

use crate::dispatch::{ModuleReg, Registrar};

#[derive(ApiModule)]
#[api_module(name = "tvm")]
pub struct TvmModule;

impl ModuleReg for TvmModule {
    fn reg(reg: &mut Registrar) {
        reg.f(execute_message, crate::tvm::execute_message::execute_api);
        reg.f(execute_get, crate::tvm::execute_get::execute_get_api);
    }
}
