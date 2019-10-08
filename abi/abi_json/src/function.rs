//! Contract function call builder.

use std::sync::Arc;
use sha2::{Digest, Sha256, Sha512};
use {Param, Token, TokenValue};
use ed25519_dalek::*;
use tvm::stack::{BuilderData, SliceData, CellData};
use ton_abi_core::types::{Bitstring, prepend_data_to_chain};
use ton_abi_core::types::{
    ABISerialized,
    ABIDeserialized,
    DeserializationError as InnerTypeDeserializationError};

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

    #[serde(skip_deserializing)]
    pub id: u32
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
    InvalidInputData(String),
    WrongVersion(u8),
    WrongId(u32)
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

    pub fn calc_function_id(signature: &str) -> u32 {
        // Sha256 hash of signature
        let mut hasher = Sha256::new();

        hasher.input(&signature.as_bytes()[..]);

        let function_hash = hasher.result();

        let mut bytes: [u8; 4] = [0; 4];
        bytes.copy_from_slice(&function_hash[..4]);

        u32::from_be_bytes(bytes)
    }

    /// Computes function ID for contract function
    pub fn get_function_id(&self) -> u32 {
        let signature = self.get_function_signature();

        //println!("{}", signature);

        Self::calc_function_id(&signature)
    }

    /// Decodes provided params from SliceData
    fn decode_params(&self, params: Vec<Param>, data: SliceData) -> Result<Vec<Token>, DeserializationError> {
        let mut tokens = vec![];

        let (version, cursor) = u8::read_from(data)
            .map_err(|err| DeserializationError::TypeDeserializationError(err))?;

        if version != ABI_VERSION { Err(DeserializationError::WrongVersion(version))? }

        let (id, mut cursor) = u32::read_from(cursor)
            .map_err(|err| DeserializationError::TypeDeserializationError(err))?;

        if id != self.id { Err(DeserializationError::WrongId(id))? }

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

    /// Parses the ABI function output to list of tokens.
    pub fn decode_output(&self, data: SliceData) -> Result<Vec<Token>, DeserializationError> {
        self.decode_params(self.output_params(), data)
    }

    /// Parses the ABI function call to list of tokens.
    pub fn decode_input(&self, mut data: SliceData) -> Result<Vec<Token>, DeserializationError> {
        data.checked_drain_reference()
            .map_err(|err| DeserializationError::InvalidInputData(err.to_string()))?;

        self.decode_params(self.input_params(), data)
    }

    /// Decodes function id from contract answer
    pub fn decode_id(data: SliceData) -> Result<u32, DeserializationError> {
        let (version, new_cursor) = u8::read_from(data)
            .map_err(|err| DeserializationError::TypeDeserializationError(err))?;

        let (id, _) = u32::read_from(new_cursor)
            .map_err(|err| DeserializationError::TypeDeserializationError(err))?;

        if version == ABI_VERSION {
            Ok(id)
        } else {
            Err(DeserializationError::WrongVersion(version))
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
            vec.extend_from_slice(&self.get_function_id().to_be_bytes()[..]);
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
        function_call: SliceData
    ) -> Result<BuilderData, DeserializationError> {
        let mut builder = BuilderData::from_slice(&function_call);

        if builder.references_free() == 0 {
            return Err(DeserializationError::InvalidInputData("No free reference for signature".to_owned()));
        }

        let mut signature = signature.to_vec();
        signature.extend_from_slice(public_key);

        let len = signature.len() * 8;

        builder.prepend_reference(BuilderData::with_raw(signature, len).unwrap());

        Ok(builder)
    }

    pub fn is_my_message(&self, data: SliceData) -> Result<bool, DeserializationError> {
        Ok(self.id == Self::decode_id(data)?)
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

        format!("{}({})", self.name, input_types)
    }

    /// Computes function ID for contract function
    pub fn get_function_id(&self) -> u32 {
        let signature = self.get_function_signature();

        //println!("{}", signature);

        Function::calc_function_id(&signature)
    }

    /// Decodes provided params from SliceData
    fn decode_params(&self, params: Vec<Param>, data: SliceData) -> Result<Vec<Token>, DeserializationError> {
        let mut tokens = vec![];

        let (version, cursor) = u8::read_from(data)
            .map_err(|err| DeserializationError::TypeDeserializationError(err))?;

        if version != ABI_VERSION { Err(DeserializationError::WrongVersion(version))? }

        let (id, mut cursor) = u32::read_from(cursor)
            .map_err(|err| DeserializationError::TypeDeserializationError(err))?;

        if id != self.id { Err(DeserializationError::WrongId(id))? }

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

    /// Parses the ABI function call to list of tokens.
    pub fn decode_input(&self, data: SliceData) -> Result<Vec<Token>, DeserializationError> {
        self.decode_params(self.input_params(), data)
    }

    /// Decodes function id from contract answer
    pub fn decode_id(data: SliceData) -> Result<u32, DeserializationError> {
        let (version, new_cursor) = u8::read_from(data)
            .map_err(|err| DeserializationError::TypeDeserializationError(err))?;

        let (id, _) = u32::read_from(new_cursor)
            .map_err(|err| DeserializationError::TypeDeserializationError(err))?;

        if version == ABI_VERSION {
            Ok(id)
        } else {
            Err(DeserializationError::WrongVersion(version))
        }
    }
}

#[cfg(test)]
#[path = "tests/test_encoding.rs"]
mod tests;
