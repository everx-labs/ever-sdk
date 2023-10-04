use crate::abi::types::Abi;
use crate::abi::Error;
use crate::boc::internal::{deserialize_cell_from_boc, serialize_cell_to_boc};
use crate::boc::BocCacheType;
use crate::boc::state_init::builder_to_cell;
use crate::client::ClientContext;
use crate::encoding::{hex_decode, slice_from_cell};
use crate::error::ClientResult;
use serde_json;
use serde_json::Value;
use std::convert::TryInto;
use std::sync::Arc;
use ton_types::{Cell, SliceData};

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ParamsOfUpdateInitialData {
    /// Contract ABI
    pub abi: Abi,
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

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ResultOfUpdateInitialData {
    /// Updated data BOC or BOC handle
    pub data: String,
}

/// Updates initial account data with initial values for the contract's static variables and owner's public key.
/// This operation is applicable only for initial account data (before deploy).
/// If the contract is already deployed, its data doesn't contain this data section any more.
/// 
/// Doesn't support ABI version >= 2.4. Use `encode_initial_data` instead
#[api_function]
pub fn update_initial_data(
    context: Arc<ClientContext>,
    params: ParamsOfUpdateInitialData,
) -> ClientResult<ResultOfUpdateInitialData> {
    let (_, mut data) = deserialize_cell_from_boc(&context, &params.data, "contract data")?;

    if !params.abi.abi()?.data_map_supported() {
        return Err(Error::invalid_abi(
            "This functionality is available only for contracts with ABI < 2.4. For ABI versions >= 2.4 use decode_account_data"
        ));
    }

    data = update_initial_data_internal(
        &params.initial_data,
        &params.abi,
        &params.initial_pubkey,
        data,
    )?;

    Ok(ResultOfUpdateInitialData {
        data: serialize_cell_to_boc(&context, data, "contract data", params.boc_cache)?,
    })
}

fn update_initial_data_internal(
    initial_data: &Option<Value>,
    abi: &Abi,
    initial_pubkey: &Option<String>,
    data: Cell,
) -> ClientResult<Cell> {
    let data = match initial_data {
        Some(init_data) => {
            let abi = abi.json_string()?;
            let data = slice_from_cell(data)?;
            ton_abi::json_abi::update_contract_data(&abi, &init_data.to_string(), data)
                .map_err(|err| Error::encode_init_data_failed(err))?
                .into_cell()
        }
        _ => data,
    };

    match initial_pubkey {
        Some(pubkey) => {
            let data = slice_from_cell(data)?;
            let pubkey = hex_decode(&pubkey)?
                .try_into()
                .map_err(|vec: Vec<u8>| Error::encode_init_data_failed(format!("invalid public key size {}", vec.len())))?;
            Ok(
                ton_abi::Contract::insert_pubkey(data, &pubkey)
                    .map_err(|err| Error::encode_init_data_failed(err))?
                    .into_cell(),
            )
        }
        _ => Ok(data),
    }
}

fn default_init_data() -> ClientResult<Cell> {
    ton_abi::Contract::insert_pubkey(Default::default(), &[0; ed25519_dalek::PUBLIC_KEY_LENGTH])
        .map_err(|err| Error::encode_init_data_failed(err))
        .map(SliceData::into_cell)
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct ParamsOfEncodeInitialData {
    /// Contract ABI
    pub abi: Abi,
    /// List of initial values for contract's static variables. `abi` parameter should be provided to set initial data
    pub initial_data: Option<Value>,
    /// Initial account owner's public key to set into account data
    pub initial_pubkey: Option<String>,

    /// Cache type to put the result.
    /// The BOC itself returned if no cache type provided.
    pub boc_cache: Option<BocCacheType>,
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ResultOfEncodeInitialData {
    /// Updated data BOC or BOC handle
    pub data: String,
}

/// Encodes initial account data with initial values for the contract's static variables and owner's
/// public key into a data BOC that can be passed to `encode_tvc` function afterwards.
///
/// This function is analogue of `tvm.buildDataInit` function in Solidity.
#[api_function]
pub fn encode_initial_data(
    context: Arc<ClientContext>,
    params: ParamsOfEncodeInitialData,
) -> ClientResult<ResultOfEncodeInitialData> {
    let abi = params.abi.abi()?;
    let data = if !abi.data_map_supported() {
        if params.initial_pubkey.is_some() {
            return Err(Error::initial_pubkey_not_supported(abi.version()));
        }
        builder_to_cell(ton_abi::json_abi::encode_storage_fields(
                &params.abi.json_string()?,
                params.initial_data.map(|data| data.to_string()).as_deref(),
            )
            .map_err(|err| Error::encode_init_data_failed(err))?
        )?
    } else {
        update_initial_data_internal(
            &params.initial_data,
            &params.abi,
            &params.initial_pubkey,
            default_init_data()?,
        )?
    };

    Ok(ResultOfEncodeInitialData {
        data: serialize_cell_to_boc(&context, data, "contract data", params.boc_cache)?,
    })
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ParamsOfDecodeInitialData {
    /// Contract ABI. Initial data is decoded if this parameter is provided
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

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ResultOfDecodeInitialData {
    /// List of initial values of contract's public variables. Initial data is decoded if `abi` input parameter is provided
    pub initial_data: Value,
    /// Initial account owner's public key
    pub initial_pubkey: String,
}

/// Decodes initial values of a contract's static variables and owner's public key from account initial data
/// This operation is applicable only for initial account data (before deploy).
/// If the contract is already deployed, its data doesn't contain this data section any more.
/// 
/// Doesn't support ABI version >= 2.4. Use `decode_account_data` instead
#[api_function]
pub fn decode_initial_data(
    context: Arc<ClientContext>,
    params: ParamsOfDecodeInitialData,
) -> ClientResult<ResultOfDecodeInitialData> {
    let (_, data) = deserialize_cell_from_boc(&context, &params.data, "contract data")?;
    let data = slice_from_cell(data)?;

    let abi = params.abi.abi()?;

    if !abi.data_map_supported() {
        return Err(Error::invalid_abi(
            "This functionality is available only for contracts with ABI < 2.4. For ABI versions >= 2.4 use decode_account_data"
        ));
    }

    let tokens = abi
        .decode_data(data.clone(), params.allow_partial)
        .map_err(|e| Error::invalid_data_for_decode(e))?;

    let initial_data = ton_abi::token::Detokenizer::detokenize_to_json_value(&tokens)
        .map_err(|e| Error::invalid_data_for_decode(e))?;

    let initial_pubkey = ton_abi::Contract::get_pubkey(&data)
        .map_err(|e| Error::invalid_data_for_decode(e))?
        .ok_or_else(|| Error::invalid_data_for_decode("no public key in contract data"))?;

    Ok(ResultOfDecodeInitialData {
        initial_data,
        initial_pubkey: hex::encode(&initial_pubkey),
    })
}
