use crate::abi::{Abi, Error};
use crate::boc::internal::{
    deserialize_cell_from_base64, deserialize_object_from_base64, serialize_object_to_base64,
};
use crate::client::ClientContext;
use crate::crypto::internal::decode_public_key;
use crate::error::ClientResult;
use crate::processing::MessageSource;
use serde_json::Value;
use std::sync::Arc;
use ton_block::GetRepresentationHash;
use ton_block::{
    Account, AccountState, AccountStorage, AccountStuff, CurrencyCollection, MsgAddressInt,
    StateInit, StateInitLib, StorageInfo, StorageUsed,
};
use ton_sdk::ContractImage;

//--------------------------------------------------------------------------------- encode_account

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub struct StateInitParams {
    abi: Abi,
    value: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub enum StateInitSource {
    /// Deploy message.
    Message(MessageSource),
    /// State init data.
    StateInit {
        /// Code BOC. Encoded with `base64`.
        code: String,
        /// Data BOC. Encoded with `base64`.
        data: String,
        /// Library BOC. Encoded with `base64`.
        library: Option<String>,
    },
    /// Content of the TVC file. Encoded with `base64`.
    Tvc {
        tvc: String,
        public_key: Option<String>,
        init_params: Option<StateInitParams>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub struct ParamsOfEncodeAccount {
    /// Source of the account state init.
    pub state_init: StateInitSource,
    /// Initial balance.
    pub balance: Option<u64>,
    /// Initial value for the `last_trans_lt`.
    pub last_trans_lt: Option<u64>,
    /// Initial value for the `last_paid`.
    pub last_paid: Option<u32>,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfEncodeAccount {
    /// Account BOC. Encoded with `base64`.
    pub account: String,
    /// Account id. Encoded with `hex`.
    pub id: String,
}

async fn state_init_from_message(
    context: &Arc<ClientContext>,
    message: &MessageSource,
) -> ClientResult<StateInit> {
    let (message, _) = message.encode(context).await?;
    let message = deserialize_object_from_base64::<ton_block::Message>(&message, "message")?.object;
    message
        .state_init()
        .map(|x| x.clone())
        .ok_or(Error::invalid_message_for_decode("missing `state_init`"))
}

fn state_init_from_bocs(
    code: &String,
    data: &String,
    library: &Option<String>,
) -> ClientResult<StateInit> {
    Ok(StateInit {
        code: Some(deserialize_cell_from_base64(code, "account code")?.1),
        data: Some(deserialize_cell_from_base64(data, "account data")?.1),
        library: if let Some(library) = library {
            StateInitLib::with_hashmap(Some(deserialize_cell_from_base64(library, "library")?.1))
        } else {
            StateInitLib::default()
        },
        split_depth: None,
        special: None,
    })
}

fn state_init_from_tvc(
    tvc: &String,
    public_key: &Option<String>,
    init_params: &Option<StateInitParams>,
) -> ClientResult<StateInit> {
    let tvc = base64::decode(tvc).map_err(|err| Error::invalid_tvc_image(err))?;
    let public_key = public_key
        .as_ref()
        .map(|x| decode_public_key(x))
        .transpose()?;

    let mut image = ContractImage::from_state_init(&mut tvc.as_slice())
        .map_err(|err| Error::invalid_message_for_decode(err))?;
    if let Some(key) = public_key {
        image
            .set_public_key(&key)
            .map_err(|err| Error::invalid_tvc_image(err))?;
    }
    if let Some(init_params) = init_params {
        image
            .update_data(
                init_params.value.to_string().as_str(),
                &init_params.abi.json_string(),
            )
            .map_err(|err| {
                Error::invalid_tvc_image(format!("Failed to set initial data: {}", err))
            })?;
    }
    Ok(image.state_init())
}

/// Encodes account state as it will be
#[api_function]
pub async fn encode_account(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfEncodeAccount,
) -> ClientResult<ResultOfEncodeAccount> {
    let state_init = match &params.state_init {
        StateInitSource::Message(message) => state_init_from_message(&context, message).await,
        StateInitSource::StateInit {
            code,
            data,
            library,
        } => state_init_from_bocs(code, data, library),
        StateInitSource::Tvc {
            tvc,
            public_key,
            init_params,
        } => state_init_from_tvc(tvc, public_key, init_params),
    }?;
    let id = state_init
        .hash()
        .map_err(|err| Error::invalid_tvc_image(err))?
        .to_hex_string();
    let account = Account::Account(AccountStuff {
        addr: MsgAddressInt::default(),
        storage: AccountStorage {
            balance: CurrencyCollection::from(params.balance.unwrap_or(100000000000)),
            last_trans_lt: params.last_trans_lt.unwrap_or(0),
            state: AccountState::AccountActive(state_init),
        },
        storage_stat: StorageInfo {
            due_payment: None,
            last_paid: params.last_paid.unwrap_or(0),
            used: StorageUsed::default(),
        },
    });
    Ok(ResultOfEncodeAccount {
        account: serialize_object_to_base64(&account, "account")?,
        id,
    })
}
