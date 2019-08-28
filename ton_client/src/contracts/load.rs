use ton_sdk::Contract;
use futures::Stream;
use types::{ApiResult, ApiError};
use crypto::keys::{account_encode, account_decode};
use client::ClientContext;

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct LoadParams {
    pub address: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct LoadResult {
    pub id: Option<String>,
    pub balanceGrams: Option<String>,
}

pub(crate) fn load(context: &mut ClientContext, params: LoadParams) -> ApiResult<LoadResult> {
    let address = params.address;
    let loaded = Contract::load(ton_sdk::AccountAddress::Short(account_decode(&address)?))
        .map_err(|err|ApiError::contracts_load_failed(err, &address))?
        .wait()
        .next();
    match loaded {
        Some(optional_contract_or_err) =>
            match optional_contract_or_err {
                Ok(optional_contract) =>
                    Ok(match optional_contract {
                        Some(contract) => make_result(contract),
                        None => EMPTY_RESULT
                    }),
                Err(err) => Err(ApiError::contracts_load_failed(err, &address))
            },
        None => Ok(EMPTY_RESULT)
    }
}

// Internals

const EMPTY_RESULT: LoadResult = LoadResult {
    id: None,
    balanceGrams: None,
};

fn make_result(contract: Contract) -> LoadResult {
    LoadResult {
        id: Some(account_encode(&contract.id())),
        balanceGrams: Some(contract.balance_grams().0.to_str_radix(10)),
    }
}

