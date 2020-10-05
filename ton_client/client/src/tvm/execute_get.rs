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
use crate::encoding::{base64_decode};
use crate::error::{ApiError, ApiResult};
use crate::tvm::execute_message::ExecutionOptions;
use crate::tvm::Error;

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfExecuteGet {
    pub account: String,
    pub function_name: String,
    pub stack: Option<Value>,
    pub execution_options: Option<ExecutionOptions>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ResultOfExecuteGet {
    pub stack: Value,
}

const DEFAULT_ADDRESS: &str = "0:0000000000000000000000000000000000000000000000000000000000000000";
const DEFAULT_BALANCE: &str = "0xffffffffffffffff";

#[api_function]
pub fn execute_get(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfExecuteGet,
) -> ApiResult<ResultOfExecuteGet> {
    let contract = Contract::from_bytes(&base64_decode(&params.account)?)
        .map_err(|err| Error::invalid_account_boc(err))?;
    let stack = contract
        .local_call_tvm_get_json(&params.function_name, params.stack.as_ref())
        .map_err(|err| ApiError::contracts_local_run_failed(err))?;
    Ok(ResultOfExecuteGet { stack })
}
