use crate::abi::types::Abi;
use crate::abi::Error;
use crate::client::ClientContext;
use crate::boc::internal::deserialize_cell_from_boc;
use crate::error::ClientResult;
use serde_json;
use serde_json::Value;
use std::sync::Arc;
use ton_abi::token::Detokenizer;

//---------------------------------------------------------------------------------- decode_message

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfDecodeAccountData {
    /// Contract ABI
    pub abi: Abi,

    /// Data BOC or BOC handle
    pub data: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfDecodeData {
    /// Decoded data as a JSON structure.
    pub data: Value,
}

/// Decodes account data using provided data BOC and ABI.
///
/// Note: this feature requires ABI 2.1 or higher.
#[api_function]
pub async fn decode_account_data(
    context: Arc<ClientContext>,
    params: ParamsOfDecodeAccountData,
) -> ClientResult<ResultOfDecodeData> {
    let (_, data) = deserialize_cell_from_boc(&context, &params.data, "contract data").await?;
    let abi = params.abi.abi()?;

    let tokens = abi.decode_storage_fields(data.into())
        .map_err(|e| Error::invalid_data_for_decode(e))?;

    let data = Detokenizer::detokenize_to_json_value(&tokens)
        .map_err(|e| Error::invalid_data_for_decode(e))?;
    Ok(ResultOfDecodeData { data })
}
