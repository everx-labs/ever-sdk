use crate::boc::{BocCacheType, internal::serialize_object_to_boc};
use crate::abi::Error;
use crate::client::ClientContext;
use crate::error::ClientResult;
use std::sync::Arc;
use ever_block::{Account, CurrencyCollection, MsgAddressInt};

//--------------------------------------------------------------------------------- encode_account

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct ParamsOfEncodeAccount {
    /// Account state init.
    pub state_init: String,
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

/// Creates account state BOC
///
#[api_function]
pub fn encode_account(
    context: Arc<ClientContext>,
    params: ParamsOfEncodeAccount,
) -> ClientResult<ResultOfEncodeAccount> {
    let state_init = crate::boc::internal::deserialize_object_from_boc(&context, &params.state_init, "Account state init")?;
    let id = state_init.cell.repr_hash();
    let address = MsgAddressInt::with_standart(None, 0, id.clone().into()).unwrap();
    let mut account = Account::with_address(address);
    account.set_balance(CurrencyCollection::from(params.balance.unwrap_or(100000000000)));
    account.try_activate_by_init_code_hash(&state_init.object, false)
        .map_err(|err| Error::invalid_tvc_image(err))?;
    account.set_last_tr_time(params.last_trans_lt.unwrap_or(0));
    Ok(ResultOfEncodeAccount {
        account: serialize_object_to_boc(&context, &account, "account", params.boc_cache)?,
        id: id.to_hex_string(),
    })
}
