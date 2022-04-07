use crate::abi::types::AbiParam;
use crate::abi::Error;
use crate::boc::internal::deserialize_cell_from_boc;
use crate::client::ClientContext;
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

/// Decodes BOC into JSON as a set of provided parameters.
///
/// Solidity functions use ABI types for [builder encoding](https://github.com/tonlabs/TON-Solidity-Compiler/blob/master/API.md#tvmbuilderstore).
/// The simplest way to decode such a BOC is to use ABI decoding.
/// ABI has it own rules for fields layout in cells so manually encoded
/// BOC can not be described in terms of ABI rules.
///
/// To solve this problem we introduce a new ABI type `Ref(<ParamType>)`
/// which allows to store `ParamType` ABI parameter in cell reference and, thus,
/// decode manually encoded BOCs. This type is available only in `decode_boc` function
/// and will not be available in ABI messages encoding until it is included into some ABI revision.
///
/// Such BOC descriptions covers most users needs. If someone wants to decode some BOC which
/// can not be described by these rules (i.e. BOC with TLB containing constructors of flags
/// defining some parsing conditions) then they can decode the fields up to fork condition,
/// check the parsed data manually, expand the parsing schema and then decode the whole BOC
/// with the full schema.

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
        params.allow_partial,
    )
    .map_err(|e| Error::invalid_data_for_decode(e))?;

    let data = Detokenizer::detokenize_to_json_value(&tokens)
        .map_err(|e| Error::invalid_data_for_decode(e))?;
    Ok(ResultOfDecodeBoc { data })
}
