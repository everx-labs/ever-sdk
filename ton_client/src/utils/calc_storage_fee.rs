/*
* Copyright 2018-2021 TON Labs LTD.
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

use crate::boc::internal::deserialize_object_from_boc;
use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::tvm::Error;
use std::sync::Arc;

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ParamsOfCalcStorageFee {
	// Account BOC or BOC cache reference
	pub account: String,
	// Time period in seconds
	pub period: u32,
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
pub struct ResultOfCalcStorageFee {
	// Storage fee over a period of time in nanotokens
	pub fee: String
}

/// Calculates storage fee for an account over a specified time period
#[api_function]
pub async fn calc_storage_fee(
    context: Arc<ClientContext>,
    params: ParamsOfCalcStorageFee,
) -> ClientResult<ResultOfCalcStorageFee> {
    let account = deserialize_object_from_boc::<ton_block::Account>(
        &context, &params.account, "account"
    )
    .await?
    .object;

    let storage = account.storage_info().ok_or(Error::invalid_account_boc("Account is None"))?;
    let addr = account.get_addr().ok_or(Error::invalid_account_boc("Account is None"))?;
    let config = crate::net::network_params::get_default_params(&context).await?.blockchain_config;

    if storage.last_paid() == 0 {
        return Err(Error::invalid_account_boc("Account `last_paid` field is not initialized"));
    }

    let fee = config.calc_storage_fee(
        storage,
        addr.is_masterchain(),
        storage.last_paid() + params.period,
    )
    .map_err(|err| Error::invalid_account_boc(format!("can not calculate storage fee: {}", err)))?;

    Ok(ResultOfCalcStorageFee {
        fee: format!("{}", fee)
    })
}