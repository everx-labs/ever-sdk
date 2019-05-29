use ed25519_dalek::*;
use serde_json::Value;
use token::{Detokenizer, Tokenizer};
use tvm::stack::{BuilderData, SliceData};
use {ABIError, Contract};

/// Encodes `parameters` for given `function` of contract described by `abi` into `BuilderData`
/// which can be used as message body for calling contract
pub fn encode_function_call(
    abi: String,
    function: String,
    parameters: String,
    pair: Option<&Keypair>,
) -> Result<BuilderData, ABIError> {
    let contract = Contract::load(abi.as_bytes())?;

    let function = contract.function(&function)?;

    let v: Value = serde_json::from_str(&parameters).map_err(|err| ABIError::SerdeError(err))?;

    let tokens = Tokenizer::tokenize_all(&function.input_params(), &v)
        .map_err(|err| ABIError::TokenizeError(err))?;

    function
        .encode_input(&tokens, pair)
        .map_err(|err| ABIError::SerializationError(err))
}

/// Decodes output parameters returned by contract function call
pub fn decode_function_response(
    abi: String,
    function: String,
    response: SliceData,
) -> Result<String, ABIError> {
    let contract = Contract::load(abi.as_bytes())?;

    let function = contract.function(&function)?;

    let tokens = function
        .decode_output(response)
        .map_err(|err| ABIError::DeserializationError(err))?;

    Detokenizer::detokenize(&function.output_params(), &tokens)
        .map_err(|err| ABIError::DetokenizeError(err))
}

#[cfg(test)]
#[path = "tests/full_stack_tests.rs"]
mod tests;
