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

use crate::dispatch::{ModuleReg, Registrar};

#[cfg(test)]
mod tests;

mod conversion;
mod errors;

pub use conversion::{
    convert_address, AddressStringFormat, ParamsOfConvertAddress,
    ResultOfConvertAddress,
};
pub use errors::{Error, ErrorCode};

/// Misc utility Functions.
#[derive(ApiModule)]
#[api_module(name = "utils")]
pub struct UtilsModule;

impl ModuleReg for UtilsModule {
    fn reg(reg: &mut Registrar) {
        reg.t::<AddressStringFormat>();
        reg.f(convert_address, conversion::convert_address_api);
    }
}
