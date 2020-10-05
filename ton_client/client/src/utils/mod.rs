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

use crate::dispatch::DispatchTable;

#[cfg(test)]
mod tests;

mod conversion;
mod errors;

pub use conversion::{
    convert_address, convert_address_method, ParamsOfConvertAddress, ResultOfConvertAddress,
    AddressStringFormat,
};
pub use errors::{Error, ErrorCode};

use api_doc::reflect::TypeInfo;

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.register_api_types(
        "utils",
        vec![
            AddressStringFormat::type_info,
        ],
    );
    handlers.call_method(convert_address_method, convert_address);
}
