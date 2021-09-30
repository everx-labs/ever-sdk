use crate::abi::types::Abi;
use crate::abi::Error;
use crate::client::ClientContext;
use crate::boc::internal::{deserialize_cell_from_boc, serialize_cell_to_boc};
use crate::boc::BocCacheType;
use crate::encoding::hex_decode;
use crate::error::ClientResult;
use serde_json;
use serde_json::Value;
use std::sync::Arc;

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfUpdateInitialData  {
    /// Contract ABI
    pub abi: Option<Abi>,
    /// Data BOC or BOC handle
    pub data: String,
    /// List of initial values for contract's public variables. `abi` parameter should be provided to set initial data
    pub initial_data: Option<Value>,
    /// Initial account owner's public key to set into account data
    pub initial_pubkey: Option<String>,

    /// Cache type to put the result.
    /// The BOC itself returned if no cache type provided.
    pub boc_cache: Option<BocCacheType>,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfUpdateInitialData {
    /// Updated data BOC or BOC handle
    pub data: String,
}

/// Updates account data with initial values for contract's public variables and owner's public key.
/// This operation is applicable only to pre-deployment contract data. Deployed contract data doesn't contain this data section
#[api_function]
pub async fn update_initial_data(
    context: Arc<ClientContext>,
    params: ParamsOfUpdateInitialData,
) -> ClientResult<ResultOfUpdateInitialData> {
    let (_, mut data) = deserialize_cell_from_boc(&context, &params.data, "contract data").await?;

    if let Some(init_data) = params.initial_data {
        let abi = params.abi
            .ok_or_else(|| Error::encode_init_data_failed("contract ABI required to set initial data"))?
            .json_string()?;
        data = ton_abi::json_abi::update_contract_data(&abi, &init_data.to_string(), data.into())
            .map_err(|err| Error::encode_init_data_failed(err))?
            .into_cell();
    }

    if let Some(pubkey) = params.initial_pubkey {
        data = ton_abi::Contract::insert_pubkey(data.into(), &hex_decode(&pubkey)?)
            .map_err(|err| Error::encode_init_data_failed(err))?
            .into_cell();
    }

    Ok(ResultOfUpdateInitialData { 
        data: serialize_cell_to_boc(&context, data, "contract data", params.boc_cache).await?
    })
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfDecodeInitialData  {
    /// Contract ABI. Initial data is decoded if this parameter is provided
    pub abi: Option<Abi>,
    /// Data BOC or BOC handle
    pub data: String,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfDecodeInitialData {
    /// List of initial values of contract's public variables. Initial data is decoded if `abi` input parameter is provided
    pub initial_data: Option<Value>,
    /// Initial account owner's public key
    pub initial_pubkey: String,
}

/// Decodes initial values for contract's public variables and owner's public key from account data
/// This operation is applicable only to pre-deployment contract data. Deployed contract data doesn't contain this data section
#[api_function]
pub async fn decode_initial_data(
    context: Arc<ClientContext>,
    params: ParamsOfDecodeInitialData,
) -> ClientResult<ResultOfDecodeInitialData> {
    let (_, data) = deserialize_cell_from_boc(&context, &params.data, "contract data").await?;
    let data: ton_types::SliceData = data.into();

    let initial_pubkey = ton_abi::Contract::get_pubkey(&data)
        .map_err(|e| Error::invalid_data_for_decode(e))?
        .ok_or_else(|| Error::invalid_data_for_decode("no public key in contract data"))?;

    let initial_data = if let Some(abi) = params.abi {
        let abi = abi.abi()?;

        let tokens = abi.decode_data(data)
            .map_err(|e| Error::invalid_data_for_decode(e))?;

        let initial_data = ton_abi::token::Detokenizer::detokenize_to_json_value(&tokens)
            .map_err(|e| Error::invalid_data_for_decode(e))?;

        Some(initial_data)
    } else {
        None
    };

    Ok(ResultOfDecodeInitialData { 
        initial_data,
        initial_pubkey: hex::encode(&initial_pubkey)
    })
}
