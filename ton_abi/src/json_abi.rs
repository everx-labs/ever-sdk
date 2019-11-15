/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use ed25519_dalek::*;
use serde_json::Value;
use token::{Detokenizer, Tokenizer};
use tvm::stack::{BuilderData, SliceData};
use {Contract, Function};
use crate::error::*;

/// Encodes `parameters` for given `function` of contract described by `abi` into `BuilderData`
/// which can be used as message body for calling contract
pub fn encode_function_call(
    abi: String,
    function: String,
    parameters: String,
    internal: bool,
    pair: Option<&Keypair>,
) -> AbiResult<BuilderData> {
    let contract = Contract::load(abi.as_bytes())?;

    let function = contract.function(&function)?;

    let v: Value = serde_json::from_str(&parameters).map_err(|err| AbiErrorKind::SerdeError(err))?;

    let tokens = Tokenizer::tokenize_all(&function.input_params(), &v)?;

    function.encode_input(&tokens, internal, pair)
}

/// Encodes `parameters` for given `function` of contract described by `abi` into `BuilderData`
/// which can be used as message body for calling contract. Message body is prepared for
/// signing. Sign should be the added by `add_sign_to_function_call` function
pub fn prepare_function_call_for_sign(
    abi: String,
    function: String,
    parameters: String,
) -> AbiResult<(BuilderData, Vec<u8>)> {
    let contract = Contract::load(abi.as_bytes())?;

    let function = contract.function(&function)?;

    let v: Value = serde_json::from_str(&parameters).map_err(|err| AbiErrorKind::SerdeError(err))?;

    let tokens = Tokenizer::tokenize_all(&function.input_params(), &v)?;

    function.create_unsigned_call(&tokens, false)
}

/// Add sign to messsage body returned by `prepare_function_call_for_sign` function
pub fn add_sign_to_function_call(
    signature: &[u8],
    public_key: &[u8],
    function_call: SliceData
) -> AbiResult<BuilderData> {
    Function::add_sign_to_encoded_input(signature, public_key, function_call)
}

/// Decodes output parameters returned by contract function call
pub fn decode_function_response(
    abi: String,
    function: String,
    response: SliceData,
    internal: bool,
) -> AbiResult<String> {
    let contract = Contract::load(abi.as_bytes())?;

    let function = contract.function(&function)?;

    let tokens = function.decode_output(response, internal)?;

    Detokenizer::detokenize(&function.output_params(), &tokens)
}

pub struct DecodedMessage {
    pub function_name: String,
    pub params: String
}

/// Decodes output parameters returned by some function call. Returns parametes and function name
pub fn decode_unknown_function_response(
    abi: String,
    response: SliceData,
    internal: bool,
) -> AbiResult<DecodedMessage> {
    let contract = Contract::load(abi.as_bytes())?;

    let result = contract.decode_output(response, internal)?;

    let output = Detokenizer::detokenize(&result.params, &result.tokens)?;

    Ok(DecodedMessage {
        function_name: result.function_name,
        params: output
    })
}

/// Decodes output parameters returned by some function call. Returns parametes and function name
pub fn decode_unknown_function_call(
    abi: String,
    response: SliceData,
    internal: bool,
) -> AbiResult<DecodedMessage> {
    let contract = Contract::load(abi.as_bytes())?;

    let result = contract.decode_input(response, internal)?;

    let input = Detokenizer::detokenize(&result.params, &result.tokens)?;

    Ok(DecodedMessage {
        function_name: result.function_name,
        params: input
    })
}

#[cfg(test)]
#[path = "tests/full_stack_tests.rs"]
mod tests;
