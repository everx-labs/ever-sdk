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

use serde_json::{Map, Value};
use ton_abi::{Contract, ParamType};

use super::types::{ExecutionOptions, ResolvedExecutionOptions};
use crate::abi::{encode_internal_message, Abi, CallSet, ParamsOfEncodeInternalMessage};
use crate::boc::internal::{deserialize_object_from_boc, serialize_object_to_boc};
use crate::boc::BocCacheType;
use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::processing::parsing::decode_output;
use crate::tvm::{Error, ResultOfRunTvm};
use ton_block::{Account, MsgAddressInt};
use crate::abi::decode_message::ResponsibleCall;
use std::str::FromStr;

const DEFAULT_ANSWER_ID: u32 = 0x12345678;

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfRunResponsible {
    /// Contract ABI for encoding and decoding messages
    pub abi: Abi,
    /// Account BOC in `base64`
    pub account: String,
    /// Function name
    pub function_name: String,
    /// Input parameters
    pub input: Option<Value>,
    /// Value passed with message
    pub value: Option<String>,
    /// Source address used for message.
    /// If value is missing then account's address will be used as a source.
    pub src_address: Option<String>,

    /// Execution options
    pub execution_options: Option<ExecutionOptions>,
    /// Cache type to put the result. The BOC itself returned if no cache type provided
    pub boc_cache: Option<BocCacheType>,
    /// Return updated account flag. Empty string is returned if the flag is `false`
    pub return_updated_account: Option<bool>,
}

/// Executes a `responsible` contract method.
///
/// Responsible method is an internal message handlers in `async` manner:
/// - the first parameter of the responsible method always is an `answerId` – the caller's
///   function selector.
/// - return statement produces response message to the caller's address with
///   the function selector equals to `answerId`.
///
#[api_function]
pub async fn run_responsible(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfRunResponsible,
) -> ClientResult<ResultOfRunTvm> {
    let abi = params.abi.abi()?;
    let mut account =
        deserialize_object_from_boc::<Account>(&context, &params.account, "account").await?;
    let options =
        ResolvedExecutionOptions::from_options(&context, params.execution_options).await?;
    if account.object.is_none() {
        return Err(Error::invalid_account_boc("Account is None"));
    }
    let address = account
        .object
        .get_addr()
        .ok_or_else(|| Error::invalid_account_boc("Missing account address"))?
        .to_string();
    let src_address = params.src_address.unwrap_or_else(|| address.clone());
    let src = MsgAddressInt::from_str(&src_address).map_err(|err|Error::invalid_account_boc(err))?;
    let mut input = params.input;
    let responsible = get_responsible(&src, &abi, &params.function_name, &mut input)?;
    let message = encode_internal_message(
        context.clone(),
        ParamsOfEncodeInternalMessage {
            abi: Some(params.abi.clone()),
            address: Some(address),
            src_address: Some(src_address),
            value: params.value.unwrap_or_else(|| "10000".to_string()),
            call_set: Some(CallSet {
                function_name: params.function_name,
                input,
                header: None,
            }),
            ..Default::default()
        },
    )
    .await?
    .message;

    let message_object = deserialize_object_from_boc(&context, &message, "message").await?;
    let messages =
        super::call_tvm::call_tvm_msg(&mut account.object, options, &message_object.object)?;

    let mut out_messages = vec![];
    for message in messages {
        out_messages.push(
            serialize_object_to_boc(&context, &message, "message", params.boc_cache.clone())
                .await?,
        );
    }

    // TODO decode Message object without converting to string
    let decoded = Some(decode_output(&context, &params.abi, out_messages.clone(), Some(&responsible)).await?);

    let account = if params.return_updated_account.unwrap_or_default() {
        serialize_object_to_boc(&context, &account.object, "account", params.boc_cache).await?
    } else {
        String::new()
    };

    Ok(ResultOfRunTvm {
        out_messages,
        account,
        decoded,
    })
}

fn get_responsible<'abi>(
    src: &'abi MsgAddressInt,
    abi: &'abi Contract,
    function_name: &str,
    input: &mut Option<Value>,
) -> ClientResult<ResponsibleCall<'abi>> {
    let function = abi
        .function(function_name)
        .map_err(|err| crate::abi::Error::invalid_abi(err))?;
    let answer_id_param = function.inputs.first().ok_or_else(|| {
        crate::abi::Error::invalid_abi(format!(
            "Function \"{}\" hasn't `answerId` parameter",
            function_name
        ))
    })?;
    match answer_id_param.kind {
        ParamType::Uint(size) if size == 32 => {}
        _ => {
            return Err(crate::abi::Error::invalid_abi(format!(
                "Function \"{}\" hasn't valid `answerId` parameter",
                function_name
            )))
        }
    }
    let answer_id = match input {
        Some(input) => {
            if let Some(value) = input.get(&answer_id_param.name) {
                if let Some(value) = value.as_u64() {
                    value as u32
                } else {
                    return Err(crate::abi::Error::invalid_abi(format!(
                        "Invalid input for `answerId` parameter. Function `{}`",
                        function_name
                    )));
                }
            } else {
                input[&answer_id_param.name] = Value::from(DEFAULT_ANSWER_ID);
                DEFAULT_ANSWER_ID
            }
        }
        None => {
            let mut input_map = Map::new();
            input_map.insert(answer_id_param.name.clone(), Value::from(DEFAULT_ANSWER_ID));
            *input = Some(Value::Object(input_map));
            DEFAULT_ANSWER_ID
        }
    };
    Ok(ResponsibleCall {
        src,
        function,
        answer_id
    })
}
