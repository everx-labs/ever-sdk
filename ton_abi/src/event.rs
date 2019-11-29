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

use {Function, Param, Token, TokenValue};
use ton_types::SliceData;
use crate::error::*;
use super::contract::ABI_VERSION;

/// Contract event specification.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Event {
    /// Event name.
    pub name: String,
    /// Event input.
    #[serde(default)]
    pub inputs: Vec<Param>,
    #[serde(default)]
    #[serde(deserialize_with = "super::function::deserialize_opt_u32_from_string")]
    pub id: Option<u32>
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

        Function::calc_function_id(&signature)
    }

    /// Returns ID for event emitting message
    pub fn get_id(&self) -> u32 {
        match self.id {
            Some(id) => id  & 0x7FFFFFFF,
            None => self.get_function_id() & 0x7FFFFFFF
        }
    }

    /// Decodes provided params from SliceData
    fn decode_params(&self, params: Vec<Param>, mut cursor: SliceData) -> AbiResult<Vec<Token>> {
        let mut tokens = vec![];
        let original = cursor.clone();

        let id = cursor.get_next_u32()?;

        if id != self.get_id() { Err(AbiErrorKind::WrongId(id))? }

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

    /// Check if message body is related to this event
    pub fn is_my_message(&self, data: SliceData, _internal: bool) -> Result<bool, AbiErrorKind> {
        let decoded_id = Self::decode_id(data)?;
        Ok(self.get_id() == decoded_id)
    }
}
