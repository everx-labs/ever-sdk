use std::convert::TryInto;
use std::sync::Arc;

use serde_json::Value;
use ton_abi::contract::MAX_SUPPORTED_VERSION;
use ton_abi::token::Tokenizer;
use ton_abi::TokenValue;

use crate::abi::{AbiParam, Error};
use crate::boc::BocCacheType;
use crate::boc::internal::serialize_cell_to_boc;
use crate::ClientContext;
use crate::error::ClientResult;

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfAbiEncodeBoc {
    /// Parameters to encode into BOC
    pub params: Vec<AbiParam>,
    /// Parameters and values as a JSON structure
    pub data: Value,
    /// Cache type to put the result. The BOC itself returned if no cache type provided
    pub boc_cache: Option<BocCacheType>
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfAbiEncodeBoc {
    /// BOC encoded as base64
    pub boc: String,
}

/// Encodes given parameters in JSON into a BOC using param types from ABI.
#[api_function]
pub async fn encode_boc(
    context: Arc<ClientContext>,
    params: ParamsOfAbiEncodeBoc,
) -> ClientResult<ResultOfAbiEncodeBoc> {
    let mut abi_params = Vec::with_capacity(params.params.len());
    for param in params.params {
        abi_params.push(param.try_into()?)
    }

    let tokens = Tokenizer::tokenize_all_params(&abi_params, &params.data)
        .map_err(|err| Error::invalid_abi(err))?;

    let builder = TokenValue::pack_values_into_chain(&tokens, Vec::new(), &MAX_SUPPORTED_VERSION)
        .map_err(|err| Error::invalid_abi(err))?;

    let cell = builder.into_cell()
        .map_err(|err| Error::invalid_abi(err))?;

    Ok(ResultOfAbiEncodeBoc {
        boc: serialize_cell_to_boc(&context, cell, "ABI params", params.boc_cache).await?,
    })
}
