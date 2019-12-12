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

//! Contract function call builder.

use chrono::prelude::*;
use sha2::{Digest, Sha256, Sha512};
use {Param, Token, TokenValue};
use ed25519_dalek::*;
use serde::de::Error;
use ton_types::{BuilderData, SliceData, Cell, IBitstring};
use crate::error::*;
use super::contract::ABI_VERSION;

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
    /// Calculated function ID
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_opt_u32_from_string")]
    pub id: Option<u32>,

    /// Set timestamp.
    #[serde(skip_deserializing)]
    pub set_time: bool,
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

        Self::calc_function_id(&signature)
    }

    /// Returns function ID
    pub fn get_id(&self) -> u32 {
        match self.id {
            Some(id) => id,
            None => self.get_function_id()
        }
    }

    /// Returns ID for call message
    pub fn get_input_id(&self) -> u32 {
        self.get_id() & 0x7FFFFFFF
    }

    /// Returns ID for response message
    pub fn get_output_id(&self) -> u32 {
        self.get_id() | 0x80000000
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
        builder = TokenValue::pack_values_into_chain(tokens, vec![builder])?;
        if !internal {
            // delete sign reference before hash
            let mut slice = SliceData::from(builder);
            slice.checked_drain_reference()?;
            builder = BuilderData::from_slice(&slice);
        }

        let hash = Cell::from(&builder).repr_hash().as_slice().to_vec();

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

    /// Check if message body is related to this function
    pub fn is_my_message(&self, data: SliceData, _internal: bool) -> Result<bool, AbiErrorKind> {
        let decoded_id = Self::decode_id(data)?;
        Ok(self.get_input_id() == decoded_id || self.get_output_id() == decoded_id)
    }
}

struct StringVisitor;

impl<'de> serde::de::Visitor<'de> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("String")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where E: serde::de::Error {
        Ok(v)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error {
        Ok(v.to_string())
    }
}

pub fn deserialize_opt_u32_from_string<'de, D>(d: D) -> Result<Option<u32>, D::Error>
    where D: serde::Deserializer<'de>
{
    match d.deserialize_string(StringVisitor) {
        Err(_) => Ok(None),
        Ok(string) => {
            if !string.starts_with("0x") {
                return Err(D::Error::custom(format!("Number parsing error: number must be prefixed with 0x ({})", string)));
            }
        
            u32::from_str_radix(&string[2..], 16)
                .map_err(|err| D::Error::custom(format!("Error parsing number: {}", err)))
                .map(|value| Some(value))
        }
    }
}

#[cfg(test)]
#[path = "tests/test_encoding.rs"]
mod tests;
