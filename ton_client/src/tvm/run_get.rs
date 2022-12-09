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

use serde_json::Value;

use super::stack;
use super::types::{ExecutionOptions, ResolvedExecutionOptions};
use crate::boc::internal::deserialize_object_from_boc;
use crate::client::ClientContext;
use crate::crypto::internal::ton_crc16;
use crate::error::ClientResult;
use crate::tvm::Error;
use std::sync::Arc;
use ton_vm::stack::integer::IntegerData;
use ton_vm::stack::{Stack, StackItem};

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfRunGet {
    /// Account BOC in `base64`
    pub account: String,
    /// Function name
    pub function_name: String,
    /// Input parameters
    pub input: Option<Value>,
    /// Execution options
    pub execution_options: Option<ExecutionOptions>,
    /// Convert lists based on nested tuples in the **result** into plain arrays. Default is `false`.
    /// Input parameters may use any of lists representations
    /// If you receive this error on Web: "Runtime error. Unreachable code should not be executed...",
    /// set this flag to true.
    /// This may happen, for example, when elector contract contains too many participants
    pub tuple_list_as_array: Option<bool>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ResultOfRunGet {
    /// Values returned by get-method on stack
    pub output: Value,
}

/// Executes a get-method of FIFT contract
///
/// Executes a get-method of FIFT contract that fulfills the smc-guidelines https://test.ton.org/smc-guidelines.txt
/// and returns the result data from TVM's stack

#[api_function]
pub async fn run_get(
    context: Arc<ClientContext>,
    params: ParamsOfRunGet,
) -> ClientResult<ResultOfRunGet> {
    let mut account: ton_block::Account =
        deserialize_object_from_boc(&context, &params.account, "account").await?.object;
    let options = ResolvedExecutionOptions::from_options(&context, params.execution_options).await?;

    if account.is_none() {
        return Err(Error::invalid_account_boc("Account is None"))
    }

    let crc = ton_crc16(params.function_name.as_bytes());
    let function_id = ((crc as u32) & 0xffff) | 0x10000;
    let mut stack_in = Stack::new();
    if let Some(input) = params.input {
        if let Value::Array(array) = input {
            for value in array {
                stack_in.push(stack::deserialize_item(&value)?);
            }
        } else {
            stack_in.push(stack::deserialize_item(&input)?);
        }
    }

    stack_in.push(StackItem::Integer(Arc::new(IntegerData::from_u32(
        function_id,
    ))));

    let engine = super::call_tvm::call_tvm(&mut account, options, stack_in)?;
    Ok(ResultOfRunGet {
        output: stack::serialize_items(
            Box::new(engine.stack().iter()),
            params.tuple_list_as_array.unwrap_or_default(),
        )?,
    })
}
