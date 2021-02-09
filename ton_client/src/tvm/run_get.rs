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

use super::stack;
use super::types::{ExecutionOptions, ResolvedExecutionOptions};
use crate::boc::internal::deserialize_object_from_boc;
use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::tvm::Error;
use std::sync::Arc;
use ton_vm::stack::integer::IntegerData;
use ton_vm::stack::{Stack, StackItem};

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfRunGet {
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
pub struct ResultOfRunGet {
    /// Values returned by getmethod on stack
    pub output: Value,
}

/// Executes a getmethod of FIFT contract 
/// 
/// Executes a getmethod of FIFT contract that fulfills the smc-guidelines https://test.ton.org/smc-guidelines.txt
/// and returns the result data from TVM's stack

#[api_function]
pub async fn run_get(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfRunGet,
) -> ClientResult<ResultOfRunGet> {
    let account: ton_block::Account =
        deserialize_object_from_boc(&context, &params.account, "account").await?.object;
    let options = ResolvedExecutionOptions::from_options(&context, params.execution_options).await?;

    let stuff = match account {
        ton_block::Account::AccountNone => Err(Error::invalid_account_boc("Acount is None")),
        ton_block::Account::Account(stuff) => Ok(stuff),
    }?;

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

    let (engine, _) = super::call_tvm::call_tvm(stuff, options, stack_in)?;
    Ok(ResultOfRunGet {
        output: stack::serialize_items(engine.stack().iter())?,
    })
}
