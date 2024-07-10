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
 *
 */

#[cfg(test)]
mod tests;

pub(crate) mod calc_storage_fee;
pub(crate) mod conversion;
#[cfg(feature = "include-zstd")]
pub(crate) mod compression;
pub(crate) mod json;
mod errors;

pub use calc_storage_fee::{
    calc_storage_fee, ParamsOfCalcStorageFee, ResultOfCalcStorageFee
};
pub use conversion::{
    convert_address, AddressStringFormat, ParamsOfConvertAddress, ResultOfConvertAddress,
    get_address_type, ParamsOfGetAddressType, ResultOfGetAddressType,
};
#[cfg(feature = "include-zstd")]
pub use compression::{compress_zstd, decompress_zstd};
pub use errors::{Error, ErrorCode};
pub use crate::encoding::AccountAddressType;
