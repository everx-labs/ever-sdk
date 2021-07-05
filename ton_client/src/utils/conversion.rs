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

use crate::client;
use crate::client::ClientContext;
use crate::encoding::{account_decode, account_encode_ex, AccountAddressType, Base64AddressParams, decode_std_base64};
use crate::error::ClientResult;
use std::sync::Arc;
use ton_block::MsgAddressInt;

use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, ApiType, Clone)]
#[serde(tag = "type")]
pub enum AddressStringFormat {
    AccountId,
    Hex,
    Base64 { url: bool, test: bool, bounce: bool },
}

impl Default for AddressStringFormat {
    fn default() -> Self {
        AddressStringFormat::Hex
    }
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ParamsOfConvertAddress {
    /// Account address in any TON format.
    pub address: String,
    /// Specify the format to convert to.
    pub output_format: AddressStringFormat,
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ResultOfConvertAddress {
    /// Address in the specified format
    pub address: String,
}

/// Converts address from any TON format to any TON format
#[api_function]
pub fn convert_address(
    _context: Arc<ClientContext>,
    params: ParamsOfConvertAddress,
) -> ClientResult<ResultOfConvertAddress> {
    let address = account_decode(&params.address)?;
    let (addr_type, base64_params) = match params.output_format {
        AddressStringFormat::Hex => (AccountAddressType::Hex, None),
        AddressStringFormat::AccountId => (AccountAddressType::AccountId, None),
        AddressStringFormat::Base64 { url, test, bounce } => (
            AccountAddressType::Base64,
            Some(Base64AddressParams { url, test, bounce }),
        ),
    };
    Ok(ResultOfConvertAddress {
        address: account_encode_ex(&address, addr_type, base64_params)?,
    })
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ParamsOfGetAddressType {
    /// Account address in any TON format.
    pub address: String,
}

#[derive(Serialize, Deserialize, ApiType, Debug)]
pub struct ResultOfGetAddressType {
    /// Account address type.
    pub address_type: AccountAddressType,
}

lazy_static! {
    static ref ACCOUNT_ID_REGEX: regex::Regex = regex::Regex::new(r#"^[0-9a-fA-F]{64}$"#).unwrap();
}

/// Returns the type of any TON address
#[api_function]
pub fn get_address_type(
    _context: Arc<ClientContext>,
    params: ParamsOfGetAddressType,
) -> ClientResult<ResultOfGetAddressType> {
    match MsgAddressInt::from_str(&params.address) {
        Ok(_address) => Ok(if ACCOUNT_ID_REGEX.is_match(&params.address) {
            AccountAddressType::AccountId
        } else {
            AccountAddressType::Hex
        }),
        Err(_err) if params.address.len() == 48 => decode_std_base64(&params.address)
            .map(|_addr| AccountAddressType::Base64),
        Err(err) => Err(client::Error::invalid_address(err, &params.address)),
    }.map(|address_type| ResultOfGetAddressType { address_type })
}

#[cfg(test)]
pub fn abi_uint(n: u128, size: usize) -> serde_json::Value {
    serde_json::to_value(ton_abi::TokenValue::Uint(ton_abi::Uint::new(n, size)))
        .unwrap_or(Default::default())
}
