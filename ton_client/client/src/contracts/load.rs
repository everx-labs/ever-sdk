/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use ton_sdk::Contract;
use crate::types::{ApiResult, ApiError};
use crate::crypto::keys::{account_decode};
use crate::client::ClientContext;

#[derive(Deserialize)]
pub(crate) struct LoadParams {
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LoadResult {
    pub id: Option<String>,
    pub balance_grams: Option<String>,
}

pub(crate) async fn load(context: &mut ClientContext, params: LoadParams) -> ApiResult<LoadResult> {
    let client = context.get_client()?;
    let loaded = Contract::load(client, &account_decode(&params.address)?)
        .await
        .map_err(|err|ApiError::contracts_load_failed(err, &params.address))?;
    match loaded {
        Some(contract) => make_result(contract),
        None => Ok(EMPTY_RESULT)
    }
}

// Internals

const EMPTY_RESULT: LoadResult = LoadResult {
    id: None,
    balance_grams: None,
};

fn make_result(contract: Contract) -> ApiResult<LoadResult> {
    Ok(LoadResult {
        id: Some(contract.id().to_hex_string()),
        balance_grams: Some(contract.balance_grams().to_string()),
    })
}
