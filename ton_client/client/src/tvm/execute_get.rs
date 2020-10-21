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
use crate::encoding::base64_decode;
use crate::error::ClientResult;
use crate::tvm::execute_message::ExecutionOptions;
use crate::tvm::Error;
use std::sync::Arc;
use ton_vm::stack::integer::IntegerData;
use ton_vm::stack::{Stack, StackItem};
use super::stack;


#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfExecuteGet {
    /// Account BOC in `base64`
    pub account: String,
    /// Function name
    pub function_name: String,
    /// Input parameters
    pub input: Option<Value>,
    pub execution_options: Option<ExecutionOptions>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ResultOfExecuteGet {
    /// Values returned by getmethod on stack
    pub output: Value,
}


/// Executes getmethod and returns data from TVM stack

#[api_function]
pub fn execute_get(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfExecuteGet,
) -> ClientResult<ResultOfExecuteGet> {
    let contract = Contract::from_bytes(&base64_decode(&params.account)?)
        .map_err(|err| Error::invalid_account_boc(err))?;

    let code = contract
        .get_code()
        .ok_or(Error::invalid_account_boc("missing required code"))?;
    let mut crc = crc_any::CRC::crc16xmodem();
    crc.digest(params.function_name.as_bytes());
    let function_id = ((crc.get_crc() as u32) & 0xffff) | 0x10000;
    let mut stack_in = Stack::new();
    if let Some(input) = params.input {
        if let Value::Array(array) = input {
            for value in array.iter() {
                stack_in.push(stack::deserialize_item(value)?);
            }
        } else {
            stack_in.push(stack::deserialize_item(&input)?);
        }
    }

    stack_in.push(StackItem::Integer(Arc::new(IntegerData::from_u32(
        function_id,
    ))));

    let stack_out = ton_sdk::call_tvm_stack(
        contract.balance,
        contract
            .balance_other_as_hashmape()
            .map_err(|err| Error::invalid_account_boc(err))?,
        &contract.id,
        None,
        (context.env.now_ms() / 1000) as u32,
        code,
        contract.get_data(),
        stack_in,
    ).map_err(|err|Error::unknown_execution_error(err))?;

    Ok(ResultOfExecuteGet {
        output: stack::serialize_items(stack_out.iter())?,
    })
}
