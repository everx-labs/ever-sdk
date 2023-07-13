use crate::abi::types::Abi;
use crate::abi::Error;
use crate::client::ClientContext;
use crate::boc::internal::deserialize_cell_from_boc;
use crate::encoding::slice_from_cell;
use crate::error::ClientResult;
use serde_json;
use serde_json::Value;
use std::sync::Arc;
use ton_abi::token::Detokenizer;

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfDecodeAccountData {
    /// Contract ABI
    pub abi: Abi,

    /// Data BOC or BOC handle
    pub data: String,

    /// Flag allowing partial BOC decoding when ABI doesn't describe the full body BOC.
    /// Controls decoder behaviour when after decoding all described in ABI params there are some data left in BOC:
    /// `true` - return decoded values
    /// `false` - return error of incomplete BOC deserialization (default)
    #[serde(default)]
    pub allow_partial: bool,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfDecodeAccountData {
    /// Decoded data as a JSON structure.
    pub data: Value,
}

/// Decodes account data using provided data BOC and ABI.
///
/// Note: this feature requires ABI 2.1 or higher.
#[api_function]
pub fn decode_account_data(
    context: Arc<ClientContext>,
    params: ParamsOfDecodeAccountData,
) -> ClientResult<ResultOfDecodeAccountData> {
    let (_, data) = deserialize_cell_from_boc(&context, &params.data, "contract data")?;
    let abi = params.abi.abi()?;

    let tokens = abi.decode_storage_fields(slice_from_cell(data)?, params.allow_partial)
        .map_err(|e| Error::invalid_data_for_decode(e))?;

    let data = Detokenizer::detokenize_to_json_value(&tokens)
        .map_err(|e| Error::invalid_data_for_decode(e))?;
    Ok(ResultOfDecodeAccountData { data })
}
