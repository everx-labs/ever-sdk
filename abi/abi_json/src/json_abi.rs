use ed25519_dalek::*;
use serde_json::Value;
use token::{Detokenizer, Tokenizer};
use tvm::stack::{BuilderData, SliceData};
use {ABIError, Contract, Function};

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

/// Encodes `parameters` for given `function` of contract described by `abi` into `BuilderData`
/// which can be used as message body for calling contract. Message body is prepared for
/// signing. Sign should be the added by `add_sign_to_function_call` function
pub fn prepare_function_call_for_sign(
    abi: String,
    function: String,
    parameters: String,
) -> Result<(BuilderData, Vec<u8>), ABIError> {
    let contract = Contract::load(abi.as_bytes())?;

    let function = contract.function(&function)?;

    let v: Value = serde_json::from_str(&parameters).map_err(|err| ABIError::SerdeError(err))?;

    let tokens = Tokenizer::tokenize_all(&function.input_params(), &v)
        .map_err(|err| ABIError::TokenizeError(err))?;

    function
        .prepare_input_for_sign(&tokens)
        .map_err(|err| ABIError::SerializationError(err))
}

/// Add sign to messsage body returned by `prepare_function_call_for_sign` function
pub fn add_sign_to_function_call(
    signature: &[u8],
    public_key: &[u8],
    function_call: SliceData
) -> Result<BuilderData, ABIError> {
    Function::add_sign_to_encoded_input(signature, public_key, function_call)
        .map_err(|err| ABIError::DeserializationError(err))
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

pub struct DecodedMessage {
    pub function_name: String,
    pub params: String
}

 /// Decodes output parameters returned by some function call. Returns parametes and function name
pub fn decode_unknown_function_call(
    abi: String,
    response: SliceData,
) -> Result<DecodedMessage, ABIError> {
    let contract = Contract::load(abi.as_bytes())?;

     let result = contract.decode_input(response)?;

     let input = Detokenizer::detokenize(&result.params, &result.tokens)
        .map_err(|err| ABIError::DetokenizeError(err))?;

     Ok(DecodedMessage {
        function_name: result.function_name,
        params: input
    })
}

#[cfg(test)]
#[path = "tests/full_stack_tests.rs"]
mod tests;
