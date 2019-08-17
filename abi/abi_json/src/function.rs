//! Contract function call builder.

use std::sync::Arc;
use sha2::{Digest, Sha256, Sha512};
use {Param, Token, TokenValue};
use ed25519_dalek::*;
use tvm::stack::{BuilderData, SliceData, CellData};
use ton_abi_core::types::{Bitstring, prepend_data_to_chain};
use ton_abi_core::types::{ABISerialized, DeserializationError as InnerTypeDeserializationError};

pub const   ABI_VERSION: u8                 = 0;
const       ABI_VERSION_BITS_SIZE: usize    = 8;
const       FUNC_ID_BITS_SIZE: usize        = 32;

/// Contract function specification.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Function {
    /// Function name.
    pub name: String,
    /// Function input.
    #[serde(default)]
    pub inputs: Vec<Param>,
    /// Function output.
    #[serde(default)]
    pub outputs: Vec<Param>,
    /// Signed function.
    #[serde(default)]
    pub signed: bool,
}

#[derive(Debug)]
pub enum SerializationError {
    WrongParameterType,
    KeyPairNeeded,
}

#[derive(Debug)]
pub enum DeserializationError {
    TypeDeserializationError(InnerTypeDeserializationError),
    IncompleteDeserializationError,
    InvalidInputData(String)
}

impl Function {
    /// Returns all input params of given function.
    pub fn input_params(&self) -> Vec<Param> {
        self.inputs.iter()
            .map(|p| p.clone())
            .collect()
    }

    /// Returns all output params of given function.
    pub fn output_params(&self) -> Vec<Param> {
        self.outputs.iter()
            .map(|p| p.clone())
            .collect()
    }

    /// Retruns ABI function signature
    pub fn get_function_signature(&self) -> String {
        let input_types = self.inputs.iter()
            .map(|param| param.kind.type_signature())
            .collect::<Vec<String>>()
            .join(",");

        let output_types = self.outputs.iter()
            .map(|param| param.kind.type_signature())
            .collect::<Vec<String>>()
            .join(",");

        format!("{}({})({})", self.name, input_types, output_types)
    }

    /// Computes function ID for contract function
    pub fn get_function_id(&self) -> [u8; 4] {
        let signature = self.get_function_signature();

        //println!("{}", signature);

        // Sha256 hash of signature
        let mut hasher = Sha256::new();

        hasher.input(&signature.into_bytes()[..]);

        let function_hash = hasher.result();

        let mut bytes = [0; 4];
        bytes.copy_from_slice(&function_hash[..4]);
        //println!("{:X?}", bytes);
        bytes
    }

    /// Parses the ABI function output to list of tokens.
    pub fn decode_output(&self, data: SliceData) -> Result<Vec<Token>, DeserializationError> {
        let params = self.output_params();

        let mut tokens = vec![];
        let mut cursor = data;

        for param in params {
            let (token_value, new_cursor) = TokenValue::read_from(&param.kind, cursor)
                .map_err(|err| DeserializationError::TypeDeserializationError(err))?;

            cursor = new_cursor;
            tokens.push(Token { name: param.name, value: token_value });
        }

        if cursor.remaining_references() != 0 || cursor.remaining_bits() != 0 {
            Err(DeserializationError::IncompleteDeserializationError)
        } else {
            Ok(tokens)
        }
    }

    /// Encodes provided function parameters into `BuilderData` containing ABI contract call
    pub fn encode_input(
        &self,
        tokens: &[Token],
        pair: Option<&Keypair>
    ) -> Result<BuilderData, SerializationError> {
        let (mut builder, hash) = self.prepare_input_for_sign(tokens)?;

        match pair {
            Some(pair) => {
                let mut signature = pair.sign::<Sha512>(&hash).to_bytes().to_vec();
                signature.extend_from_slice(&pair.public.to_bytes());
    
                let len = signature.len() * 8;

                builder.prepend_reference(BuilderData::with_raw(signature, len).unwrap());
            },
            None => builder.prepend_reference(BuilderData::new())
        }

        Ok(builder)
    }

    /// Encodes provided function parameters into `BuilderData` containing ABI contract call.
    /// `BuilderData` is prepared for signing. Sign should be the added by `add_sign_to_function_call` function
    pub fn prepare_input_for_sign(
        &self,
        tokens: &[Token]
    ) -> Result<(BuilderData, Vec<u8>), SerializationError> {
        let params = self.input_params();

        if !Token::types_check(tokens, params.as_slice()) {
            return Err(SerializationError::WrongParameterType);
        }

        // prepare standard message
        let mut builder = BuilderData::new();
        for token in tokens.iter().rev() {
            builder = token.value.prepend_to(builder);
            //println!("{}", builder);
        }

        // expand cells chain with new root if all references are used 
        // or if ABI version and function ID cannot fit into root cell
        if  BuilderData::references_capacity() == builder.references_used() ||
            BuilderData::bits_capacity() < builder.bits_used() + FUNC_ID_BITS_SIZE + ABI_VERSION_BITS_SIZE
        {
            let mut new_builder = BuilderData::new();
            new_builder.append_reference(builder);
            builder = new_builder;
        };        

        builder = prepend_data_to_chain(builder, {
            // make prefix with ABI version and function ID
            let mut vec = vec![ABI_VERSION];
            vec.extend_from_slice(&self.get_function_id()[..]);
            let len = vec.len() * 8;
            Bitstring::create(vec, len)
        });

        let hash = (&Arc::<CellData>::from(&builder)).repr_hash().as_slice().to_vec();

        Ok((builder, hash))
    }

    /// Add sign to messsage body returned by `prepare_input_for_sign` function
    pub fn add_sign_to_encoded_input(
        signature: &[u8],
        public_key: &[u8],
        mut function_call: SliceData
    ) -> Result<BuilderData, DeserializationError> {
        if 0 == function_call.remaining_references() {
             return Err(DeserializationError::InvalidInputData("No signature cell".to_owned()));
        }

        let signature_cell = function_call.checked_drain_reference().unwrap();

        if 0 != signature_cell.calc_bit_length() {
             return Err(DeserializationError::InvalidInputData("Signature cell is not empty".to_owned()));
        }

        let mut builder = BuilderData::from_slice(&function_call);

        let mut signature = signature.to_vec();
        signature.extend_from_slice(public_key);

        let len = signature.len() * 8;

        builder.prepend_reference(BuilderData::with_raw(signature, len).unwrap());

        Ok(builder)
    }
}

#[cfg(test)]
#[path = "tests/test_encoding.rs"]
mod tests;
