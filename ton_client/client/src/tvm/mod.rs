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

use serde_json::Value;

use ton_sdk::Contract;

use crate::client::ClientContext;
use crate::types::{ApiResult, ApiError};
use crate::dispatch::DispatchTable;

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfLocalRunGet {
    pub codeBase64: Option<String>,
    pub dataBase64: Option<String>,
    pub functionName: String,
    pub input: Option<Value>,
    pub address: Option<String>,
    pub balance: Option<String>,
    pub last_paid: Option<u32>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfLocalRunGet {
    pub output: Value,
}

const DEFAULT_ADDRESS: &str = "0:0000000000000000000000000000000000000000000000000000000000000000";
const DEFAULT_BALANCE: &str = "0xffffffffffffffff";

pub(crate) fn get(
    context: &mut ClientContext,
    params: ParamsOfLocalRunGet,
) -> ApiResult<ResultOfLocalRunGet> {
    debug!("-> contracts.run.get({})",
        params.functionName,
    );

    let contract = match &params.codeBase64 {
        // load contract data from node manually
        #[cfg(feature = "node_interaction")]
        None => {
            debug!("load contract");
            let address = params.address.ok_or_else(|| ApiError::address_reqired_for_runget())?;
            let address = crate::crypto::keys::account_decode(&address)?;
            let mut runtime = context.take_runtime()?;
            let result = runtime.block_on(crate::contracts::run::load_contract(context, &address));
            context.runtime = Some(runtime);
            result?
        }
        // can't load
        #[cfg(not(feature = "node_interaction"))]
        None => {
            debug!("no account provided");
            let _context = context;
            return Err(ApiError::invalid_params("", "No account provided"));
        }

        Some(code) => {
            let last_paid = params.last_paid.unwrap_or(Contract::now());
            let contract_json = json!({
                "id": params.address.unwrap_or(DEFAULT_ADDRESS.to_string()),
                "acc_type": 1,
                "balance": params.balance.unwrap_or(DEFAULT_BALANCE.to_string()),
                "code": code,
                "data": params.dataBase64,
                "last_paid": last_paid,
            });
            Contract::from_json(contract_json.to_string().as_str())
                .map_err(|err| ApiError::contracts_local_run_failed(err))?
        }
    };

    let output = contract.local_call_tvm_get_json(
        &params.functionName,
        params.input.as_ref()
    ).map_err(|err| ApiError::contracts_local_run_failed(err))?;
    Ok(ResultOfLocalRunGet { output })
}

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.spawn("tvm.get", get);
}
