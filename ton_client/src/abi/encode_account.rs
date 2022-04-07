use crate::abi::{Abi, Error};
use crate::boc::internal::{deserialize_cell_from_boc, deserialize_object_from_boc};
use crate::client::ClientContext;
use crate::crypto::internal::decode_public_key;
use crate::error::ClientResult;
use crate::{
    abi::types::MessageSource,
    boc::{internal::serialize_object_to_boc, BocCacheType},
};
use serde_json::Value;
use std::sync::Arc;
use ton_block::GetRepresentationHash;
use ton_block::{Account, CurrencyCollection, MsgAddressInt, StateInit, StateInitLib};
use ton_sdk::ContractImage;

//--------------------------------------------------------------------------------- encode_account

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct StateInitParams {
    abi: Abi,
    value: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
#[serde(tag = "type")]
pub enum StateInitSource {
    /// Deploy message.
    Message { source: MessageSource },
    /// State init data.
    StateInit {
        /// Code BOC. Encoded in `base64`.
        code: String,
        /// Data BOC. Encoded in `base64`.
        data: String,
        /// Library BOC. Encoded in `base64`.
        library: Option<String>,
    },
    /// Content of the TVC file. Encoded in `base64`.
    Tvc {
        tvc: String,
        public_key: Option<String>,
        init_params: Option<StateInitParams>,
    },
}

impl Default for StateInitSource {
    fn default() -> Self {
        StateInitSource::Tvc {
            tvc: Default::default(),
            public_key: Default::default(),
            init_params: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct ParamsOfEncodeAccount {
    /// Source of the account state init.
    pub state_init: StateInitSource,
    /// Initial balance.
    pub balance: Option<u64>,
    /// Initial value for the `last_trans_lt`.
    pub last_trans_lt: Option<u64>,
    /// Initial value for the `last_paid`.
    pub last_paid: Option<u32>,
    /// Cache type to put the result. The BOC itself returned if no cache type provided
    pub boc_cache: Option<BocCacheType>,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfEncodeAccount {
    /// Account BOC encoded in `base64`.
    pub account: String,
    /// Account ID  encoded in `hex`.
    pub id: String,
}

async fn state_init_from_message(
    context: &Arc<ClientContext>,
    message: &MessageSource,
) -> ClientResult<StateInit> {
    let (message, _) = message.encode(context).await?;
    let message = deserialize_object_from_boc::<ton_block::Message>(context, &message, "message")
        .await?
        .object;
    message
        .state_init()
        .map(|x| x.clone())
        .ok_or(Error::invalid_message_for_decode("missing `state_init`"))
}

async fn state_init_from_bocs(
    context: &ClientContext,
    code: &String,
    data: &String,
    library: &Option<String>,
) -> ClientResult<StateInit> {
    Ok(StateInit {
        code: Some(
            deserialize_cell_from_boc(context, code, "account code")
                .await?
                .1,
        ),
        data: Some(
            deserialize_cell_from_boc(context, data, "account data")
                .await?
                .1,
        ),
        library: if let Some(library) = library {
            StateInitLib::with_hashmap(Some(
                deserialize_cell_from_boc(context, library, "library")
                    .await?
                    .1,
            ))
        } else {
            StateInitLib::default()
        },
        split_depth: None,
        special: None,
    })
}

async fn state_init_from_tvc(
    context: &ClientContext,
    tvc: &String,
    public_key: &Option<String>,
    init_params: &Option<StateInitParams>,
) -> ClientResult<StateInit> {
    let (_, cell) = deserialize_cell_from_boc(context, tvc, "TVC image").await?;
    let public_key = public_key
        .as_ref()
        .map(|x| decode_public_key(x))
        .transpose()?;

    let mut image = ContractImage::from_cell(cell).map_err(|err| Error::invalid_tvc_image(err))?;
    if let Some(key) = public_key {
        image
            .set_public_key(&key)
            .map_err(|err| Error::invalid_tvc_image(err))?;
    }
    if let Some(init_params) = init_params {
        image
            .update_data(
                init_params.value.to_string().as_str(),
                &init_params.abi.json_string()?,
            )
            .map_err(|err| {
                Error::invalid_tvc_image(format!("Failed to set initial data: {}", err))
            })?;
    }
    Ok(image.state_init())
}

/// Creates account state BOC
///
/// Creates account state provided with one of these sets of data :
/// 1. BOC of code, BOC of data, BOC of library
/// 2. TVC (string in `base64`), keys, init params
#[api_function]
pub async fn encode_account(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfEncodeAccount,
) -> ClientResult<ResultOfEncodeAccount> {
    let state_init = match &params.state_init {
        StateInitSource::Message { source } => state_init_from_message(&context, source).await,
        StateInitSource::StateInit {
            code,
            data,
            library,
        } => state_init_from_bocs(&context, code, data, library).await,
        StateInitSource::Tvc {
            tvc,
            public_key,
            init_params,
        } => state_init_from_tvc(&context, tvc, public_key, init_params).await,
    }?;
    let id = state_init
        .hash()
        .map_err(|err| Error::invalid_tvc_image(err))?;
    let address = MsgAddressInt::with_standart(None, 0, id.clone().into()).unwrap();
    let mut account = Account::with_address(address);
    account.set_balance(CurrencyCollection::from(
        params.balance.unwrap_or(100000000000),
    ));
    account
        .try_activate_by_init_code_hash(&state_init, false)
        .map_err(|err| Error::invalid_tvc_image(err))?;
    account.set_last_tr_time(params.last_trans_lt.unwrap_or(0));
    Ok(ResultOfEncodeAccount {
        account: serialize_object_to_boc(&context, &account, "account", params.boc_cache).await?,
        id: id.as_hex_string(),
    })
}
