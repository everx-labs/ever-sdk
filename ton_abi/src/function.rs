//! Contract function call builder.

use std::sync::Arc;
use chrono::prelude::*;
use sha2::{Digest, Sha256, Sha512};
use {Param, Token, TokenValue};
use ed25519_dalek::*;
use tvm::stack::{BuilderData, SliceData, CellData, IBitstring};
use crate::error::*;

pub const   ABI_VERSION: u8 = 1;

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
    #[serde(skip_deserializing)]
    pub set_time: bool,
    /// Calculated function ID
    #[serde(skip_deserializing)]
    pub id: u32
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

    /// Returns true if function has input parameters, false in not
    pub fn has_input(&self) -> bool {
        self.inputs.len() != 0
    }

    /// Returns true if function has output parameters, false in not
    pub fn has_output(&self) -> bool {
        self.outputs.len() != 0
    }

    /// Retruns ABI function signature
    pub fn get_function_signature(&self) -> String {
        let mut input_types = self.inputs.iter()
            .map(|param| param.kind.type_signature())
            .collect::<Vec<String>>();

        if self.set_time {
            input_types.insert(0, "time".to_owned())
        }
        
        let input_types = input_types.join(",");

        let output_types = self.outputs.iter()
            .map(|param| param.kind.type_signature())
            .collect::<Vec<String>>()
            .join(",");

        format!("{}({})({})v{}", self.name, input_types, output_types, ABI_VERSION)
    }

    pub fn calc_function_id(signature: &str) -> u32 {
        // Sha256 hash of signature
        let mut hasher = Sha256::new();

        hasher.input(&signature.as_bytes());

        let function_hash = hasher.result();

        let mut bytes: [u8; 4] = [0; 4];
        bytes.copy_from_slice(&function_hash[..4]);
        //println!("{}: {:X}", signature, u32::from_be_bytes(bytes));

        u32::from_be_bytes(bytes)
    }

    /// Computes function ID for contract function
    pub fn get_function_id(&self) -> u32 {
        let signature = self.get_function_signature();

        //println!("{}", signature);

        Self::calc_function_id(&signature)
    }

    /// Returns ID for call message
    pub fn get_input_id(&self) -> u32 {
        self.id & 0x7FFFFFFF
    }

    /// Returns ID for response message
    pub fn get_output_id(&self) -> u32 {
        self.id | 0x80000000
    }

    /// Decodes provided params from SliceData
    fn decode_params(&self, params: Vec<Param>, mut cursor: SliceData, expected_id: u32, exctract_time: bool
        ) -> AbiResult<Vec<Token>> {
        let mut tokens = vec![];
        let original = cursor.clone();

        let id = cursor.get_next_u32()?;

        if id != expected_id { Err(AbiErrorKind::WrongId(id))? }

        if exctract_time {
            cursor.get_next_u64()?;
        }

        for param in params {
            // println!("{:?}", param);
            let (token_value, new_cursor) = TokenValue::read_from(&param.kind, cursor)?;

            cursor = new_cursor;
            tokens.push(Token { name: param.name, value: token_value });
        }

        if cursor.remaining_references() != 0 || cursor.remaining_bits() != 0 {
            bail!(AbiErrorKind::IncompleteDeserializationError(original))
        } else {
            Ok(tokens)
        }
    }

    /// Parses the ABI function output to list of tokens.
    pub fn decode_output(&self, data: SliceData, _internal: bool) -> AbiResult<Vec<Token>> {
        self.decode_params(self.output_params(), data, self.get_output_id(), false)
    }

    /// Parses the ABI function call to list of tokens.
    pub fn decode_input(&self, mut data: SliceData, internal: bool) -> AbiResult<Vec<Token>> {
        if !internal {
            data.checked_drain_reference()
                .map_err(|err| AbiErrorKind::InvalidInputData(err.to_string()))?;
        }

        self.decode_params(self.input_params(), data, self.get_input_id(), self.set_time && !internal)
    }

    /// Decodes function id from contract answer
    pub fn decode_id(mut data: SliceData) -> AbiResult<u32> {
        Ok(data.get_next_u32()?)
    }

    /// Encodes provided function parameters into `BuilderData` containing ABI contract call
    pub fn encode_input(
        &self,
        tokens: &[Token],
        internal: bool,
        pair: Option<&Keypair>
    ) -> AbiResult<BuilderData> {
        let (mut builder, hash) = self.create_unsigned_call(tokens, internal)?;

        if !internal {
            match pair {
                Some(pair) => {
                    let mut signature = pair.sign::<Sha512>(&hash).to_bytes().to_vec();
                    signature.extend_from_slice(&pair.public.to_bytes());
        
                    let len = signature.len() * 8;

                    builder.prepend_reference(BuilderData::with_raw(signature, len).unwrap());
                },
                None => builder.prepend_reference(BuilderData::new())
            }
        }

        Ok(builder)
    }

    /// Encodes provided function parameters into `BuilderData` containing ABI contract call.
    /// `BuilderData` is prepared for signing. Sign should be the added by `add_sign_to_function_call` function
    pub fn create_unsigned_call(
        &self,
        tokens: &[Token],
        internal: bool
    ) -> AbiResult<(BuilderData, Vec<u8>)> {
        let params = self.input_params();

        if !Token::types_check(tokens, params.as_slice()) {
            bail!(AbiErrorKind::WrongParameterType);
        }

        // prepare standard message
        let mut builder = BuilderData::new();
        builder.append_u32(self.get_input_id())?;

        if !internal {
            if self.set_time {
                let time = Utc::now().timestamp_millis();
                builder.append_i64(time)?;
            }
            
            // reserve reference for sign
            builder.append_reference(BuilderData::new().into());
        }

        // encoding itself
        let mut builder = TokenValue::pack_values_into_chain(tokens, vec![builder])?;
        
        if !internal {
            // delete sign reference before hash
            builder.update_cell(|_, _, refs, _| {
                            refs.remove(0)
                        },
                        ());
        }

        let hash = (&Arc::<CellData>::from(&builder)).repr_hash().as_slice().to_vec();

        Ok((builder, hash))
    }

    /// Add sign to messsage body returned by `prepare_input_for_sign` function
    pub fn add_sign_to_encoded_input(
        signature: &[u8],
        public_key: &[u8],
        function_call: SliceData
    ) -> AbiResult<BuilderData> {
        let mut builder = BuilderData::from_slice(&function_call);

        if builder.references_free() == 0 {
            bail!(AbiErrorKind::InvalidInputData("No free reference for signature".to_owned()));
        }

        let mut signature = signature.to_vec();
        signature.extend_from_slice(public_key);

        let len = signature.len() * 8;

        builder.prepend_reference(BuilderData::with_raw(signature, len).unwrap());

        Ok(builder)
    }

    pub fn is_my_message(&self, data: SliceData, _internal: bool) -> Result<bool, AbiErrorKind> {
        let decoded_id = Self::decode_id(data)?;
        Ok(self.get_input_id() == decoded_id || self.get_output_id() == decoded_id)
    }
}


/// Contract event specification.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Event {
    /// Event name.
    pub name: String,
    /// Event input.
    #[serde(default)]
    pub inputs: Vec<Param>,

    #[serde(skip_deserializing)]
    pub id: u32
}


impl Event {
    /// Returns all input params of given function.
    pub fn input_params(&self) -> Vec<Param> {
        self.inputs.iter()
            .map(|p| p.clone())
            .collect()
    }

    /// Returns true if function has input parameters, false in not
    pub fn has_input(&self) -> bool {
        self.inputs.len() != 0
    }

    /// Retruns ABI function signature
    pub fn get_function_signature(&self) -> String {
        let input_types = self.inputs.iter()
            .map(|param| param.kind.type_signature())
            .collect::<Vec<String>>()
            .join(",");

        format!("{}({})v{}", self.name, input_types, ABI_VERSION)
    }

    /// Computes function ID for contract function
    pub fn get_function_id(&self) -> u32 {
        let signature = self.get_function_signature();

        //println!("{}", signature);

        Function::calc_function_id(&signature)
    }

    /// Decodes provided params from SliceData
    fn decode_params(&self, params: Vec<Param>, mut cursor: SliceData) -> AbiResult<Vec<Token>> {
        let mut tokens = vec![];
        let original = cursor.clone();

        let id = cursor.get_next_u32()?;

        if id != self.id { Err(AbiErrorKind::WrongId(id))? }

        for param in params {
            let (token_value, new_cursor) = TokenValue::read_from(&param.kind, cursor)?;

            cursor = new_cursor;
            tokens.push(Token { name: param.name, value: token_value });
        }

        if cursor.remaining_references() != 0 || cursor.remaining_bits() != 0 {
            bail!(AbiErrorKind::IncompleteDeserializationError(original))
        } else {
            Ok(tokens)
        }
    }

    /// Parses the ABI function call to list of tokens.
    pub fn decode_input(&self, data: SliceData) -> AbiResult<Vec<Token>> {
        self.decode_params(self.input_params(), data)
    }

    /// Decodes function id from contract answer
    pub fn decode_id(mut data: SliceData) -> AbiResult<u32> {
        Ok(data.get_next_u32()?)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DataItem {
    pub key: u64,
    #[serde(flatten)]
    pub value: Param,
}

#[cfg(test)]
#[path = "tests/test_encoding.rs"]
mod tests;
