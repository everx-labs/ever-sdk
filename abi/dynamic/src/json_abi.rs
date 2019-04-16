use {Contract, ABIError};
use token::Tokenizer;
use serde_json::Value;
use tvm::stack::{BuilderData, SliceData};
use ed25519_dalek::*;

pub fn encode_function_call(abi: String, function: String, parameters: String, pair: Option<&Keypair>) -> Result<BuilderData, ABIError> {
    let contract = Contract::load(abi.as_bytes())?;

    let function = contract.function(&function)?;

    let v: Value = serde_json::from_str(&parameters).map_err(|err| ABIError::SerdeError(err))?;

    let tokens = Tokenizer::tokenize_all(&function.input_params(), &v).map_err(|err| ABIError::TokenizeError(err))?;

    function.encode_input(&tokens, pair).map_err(|err| ABIError::SerializationError(err))
}

pub fn decode_function_responce(abi: String, function: String, responce: SliceData) -> Result<String, ABIError> {
    let contract = Contract::load(abi.as_bytes())?;

    let function = contract.function(&function)?;

    let tokens = function.decode_output(responce).map_err(|err| ABIError::DeserializationError(err))?;

    Err(ABIError::NotImplemented)
}
