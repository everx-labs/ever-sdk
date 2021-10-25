use crate::abi::types::AbiParam;
use crate::abi::Error;
use crate::client::ClientContext;
use crate::boc::internal::deserialize_cell_from_boc;
use crate::error::ClientResult;
use serde_json;
use serde_json::Value;
use std::convert::TryInto;
use std::sync::Arc;
use ton_abi::token::Detokenizer;

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfDecodeBoc {
    /// Parameters to decode from BOC
    pub params: Vec<AbiParam>,
    /// Data BOC or BOC handle
    pub boc: String,
    // Do not check if all BOC data is parsed by provided parameters set
    // Set it to `true` if don't need to decode the whole BOC data or if you need
    // to handle conditional parsing (when TLB constructor or flags should be 
    // checked to decide how to parse remaining BOC data)
    pub allow_partial: bool,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfDecodeBoc {
    /// Decoded data as a JSON structure.
    pub data: Value,
}

/// Decodes BOC as a set of provided parameters.
#[api_function]
pub async fn decode_boc(
    context: Arc<ClientContext>,
    params: ParamsOfDecodeBoc,
) -> ClientResult<ResultOfDecodeBoc> {
    let (_, data) = deserialize_cell_from_boc(&context, &params.boc, "").await?;

    let mut abi_params = Vec::with_capacity(params.params.len());
    for param in params.params {
        abi_params.push(param.try_into()?)
    }

    let tokens = ton_abi::TokenValue::decode_params(
        &abi_params,
        data.into(),
        &ton_abi::contract::MAX_SUPPORTED_VERSION,
        params.allow_partial
    )
        .map_err(|e| Error::invalid_data_for_decode(e))?;

    let data = Detokenizer::detokenize_to_json_value(&tokens)
        .map_err(|e| Error::invalid_data_for_decode(e))?;
    Ok(ResultOfDecodeBoc { data })
}
