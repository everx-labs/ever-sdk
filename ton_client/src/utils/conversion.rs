use crate::client::ClientContext;
use crate::encoding::{account_decode, account_encode_ex, AccountAddressType, Base64AddressParams};
use crate::error::ClientResult;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, ApiType, Clone)]
#[serde(tag = "type")]
pub enum AddressStringFormat {
    AccountId,
    Hex,
    Base64 { url: bool, test: bool, bounce: bool },
}

#[derive(Serialize, Deserialize, ApiType, Debug)]
pub struct ParamsOfConvertAddress {
    /// Account address in any TON format.
    pub address: String,
    /// Specify the format to convert to.
    pub output_format: AddressStringFormat,
}

#[derive(Serialize, Deserialize, ApiType, Debug)]
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

#[cfg(test)]
pub fn abi_uint(n: u128, size: usize) -> serde_json::Value {
    serde_json::to_value(ton_abi::TokenValue::Uint(ton_abi::Uint::new(n, size)))
        .unwrap_or(Default::default())
}
