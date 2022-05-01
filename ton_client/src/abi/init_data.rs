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
use ton_types::Cell;

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfUpdateInitialData  {
    /// Contract ABI
    pub abi: Option<Abi>,
    /// Data BOC or BOC handle
    pub data: String,
    /// List of initial values for contract's static variables. `abi` parameter should be provided to set initial data
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

/// Updates initial account data with initial values for the contract's static variables and owner's public key.
/// This operation is applicable only for initial account data (before deploy). 
/// If the contract is already deployed, its data doesn't contain this data section any more.
#[api_function]
pub async fn update_initial_data(
    context: Arc<ClientContext>,
    params: ParamsOfUpdateInitialData,
) -> ClientResult<ResultOfUpdateInitialData> {
    let (_, mut data) = deserialize_cell_from_boc(&context, &params.data, "contract data").await?;

    data = update_initial_data_internal(&params.initial_data, &params.abi, &params.initial_pubkey, data)?;

    Ok(ResultOfUpdateInitialData {
        data: serialize_cell_to_boc(&context, data, "contract data", params.boc_cache).await?
    })
}

fn update_initial_data_internal(
    initial_data: &Option<Value>,
    abi: &Option<Abi>,
    initial_pubkey: &Option<String>,
    data: Cell,
) -> ClientResult<Cell> {
    let data = match initial_data {
        Some(init_data) => {
            let abi = abi.as_ref()
                .ok_or_else(|| Error::encode_init_data_failed("contract ABI required to set initial data"))?
                .json_string()?;
            ton_abi::json_abi::update_contract_data(&abi, &init_data.to_string(), data.into())
                .map_err(|err| Error::encode_init_data_failed(err))?
                .into_cell()
        }
        _ => data
    };

    match initial_pubkey {
        Some(pubkey) => {
            Ok(ton_abi::Contract::insert_pubkey(data.into(), &hex_decode(&pubkey)?)
                .map_err(|err| Error::encode_init_data_failed(err))?
                .into_cell())
        }
        _ => Ok(data)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct ParamsOfEncodeInitialData {
    /// Contract ABI
    pub abi: Option<Abi>,
    /// List of initial values for contract's static variables. `abi` parameter should be provided to set initial data
    pub initial_data: Option<Value>,
    /// Initial account owner's public key to set into account data
    pub initial_pubkey: Option<String>,

    /// Cache type to put the result.
    /// The BOC itself returned if no cache type provided.
    pub boc_cache: Option<BocCacheType>,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfEncodeInitialData {
    /// Updated data BOC or BOC handle
    pub data: String,
}

/// Encodes initial account data with initial values for the contract's static variables and owner's
/// public key into a data BOC that can be passed to `encode_tvc` function afterwards.
///
/// This function is analogue of `tvm.buildDataInit` function in Solidity.
#[api_function]
pub async fn encode_initial_data(
    context: Arc<ClientContext>,
    params: ParamsOfEncodeInitialData,
) -> ClientResult<ResultOfEncodeInitialData> {
    let data = update_initial_data_internal(
        &params.initial_data,
        &params.abi,
        &params.initial_pubkey,
        Cell::default(),
    )?;

    Ok(ResultOfEncodeInitialData {
        data: serialize_cell_to_boc(&context, data, "contract data", params.boc_cache).await?
    })
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ParamsOfDecodeInitialData  {
    /// Contract ABI. Initial data is decoded if this parameter is provided
    pub abi: Option<Abi>,
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
pub struct ResultOfDecodeInitialData {
    /// List of initial values of contract's public variables. Initial data is decoded if `abi` input parameter is provided
    pub initial_data: Option<Value>,
    /// Initial account owner's public key
    pub initial_pubkey: String,
}

/// Decodes initial values of a contract's static variables and owner's public key from account initial data
/// This operation is applicable only for initial account data (before deploy). 
/// If the contract is already deployed, its data doesn't contain this data section any more.
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

        let tokens = abi.decode_data(data, params.allow_partial)
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
