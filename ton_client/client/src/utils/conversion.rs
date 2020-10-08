use crate::client::ClientContext;
use crate::encoding::{account_decode, account_encode_ex, AccountAddressType, Base64AddressParams};
use crate::error::ClientResult;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, ApiType, Clone)]
pub enum AddressStringFormat {
    AccountId {},
    Hex {},
    Base64 { url: bool, test: bool, bounce: bool },
}

#[derive(Serialize, Deserialize, ApiType, Debug)]
pub struct ParamsOfConvertAddress {
    /// Account address in any format.
    pub address: String,
    /// Specify the format to convert to.
    pub output_format: AddressStringFormat,
}

#[derive(Serialize, Deserialize, ApiType, Debug)]
pub struct ResultOfConvertAddress {
    /// address in the specified format
    pub address: String,
}

/// Sends message to the network and monitors network for a result of
/// message processing.
#[api_function]
pub fn convert_address(
    _context: Arc<ClientContext>,
    params: ParamsOfConvertAddress,
) -> ClientResult<ResultOfConvertAddress> {
    let address = account_decode(&params.address)?;
    let (addr_type, base64_params) = match params.output_format {
        AddressStringFormat::Hex {} => (AccountAddressType::Hex, None),
        AddressStringFormat::AccountId {} => (AccountAddressType::AccountId, None),
        AddressStringFormat::Base64 { url, test, bounce } => (
            AccountAddressType::Base64,
            Some(Base64AddressParams { url, test, bounce }),
        ),
    };
    Ok(ResultOfConvertAddress {
        address: account_encode_ex(&address, addr_type, base64_params)?,
    })
}
