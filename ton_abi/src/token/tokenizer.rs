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

//! ABI param and parsing for it.
use {ParamType, Param, Uint, Int, Token, TokenValue};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Cursor;
use num_bigint::{Sign, BigInt, BigUint};
use tvm::block::{Grams, MsgAddress};
use tvm::cells_serialization::deserialize_tree_of_cells;
use crate::error::*;
use std::str::FromStr;

/// This struct should be used to parse string values as tokens.
pub struct Tokenizer;

impl Tokenizer {
    /// Tries to parse a JSON value as a token of given type.
    pub fn tokenize_parameter(param: &ParamType, value: &Value) -> AbiResult<TokenValue> {
        match param {
            ParamType::Unknown => bail!(AbiErrorKind::NotImplemented),
            ParamType::Uint(size) => Self::tokenize_uint(*size, value),
            ParamType::Int(size) => Self::tokenize_int(*size, value),
            ParamType::Bool => Self::tokenize_bool(value),
            ParamType::Tuple(tuple_params) => Self::tokenize_tuple(tuple_params, value),
            ParamType::Array(param_type) => Self::tokenize_array(&param_type, value),
            ParamType::FixedArray(param_type, size) => Self::tokenize_fixed_array(&param_type, *size, value),
            ParamType::Cell => Self::tokenize_cell(value),
            ParamType::Map(key_type, value_type) => Self::tokenize_hashmap(key_type, value_type, value),
            ParamType::Address => {
                let address = MsgAddress::from_str(
                    &value.as_str()
                        .ok_or(AbiErrorKind::WrongDataFormat(value.clone()))?)
                    .map_err(|_| AbiErrorKind::WrongDataFormat(value.clone()))?;
                Ok(TokenValue::Address(address))
            }
            ParamType::Bytes => Self::tokenize_bytes(value, None),
            ParamType::FixedBytes(size) => Self::tokenize_bytes(value, Some(*size)),
            ParamType::Gram => Self::tokenize_gram(value),
        }
    }

    /// Tries to parse parameters from JSON values to tokens.
    pub fn tokenize_all(params: &[Param], values: &Value) -> AbiResult<Vec<Token>> {
        if let Value::Object(map) = values {
            if map.len() != params.len() {
                bail!(AbiErrorKind::WrongParametersCount(params.len(), map.len()))
            }

            let mut tokens = Vec::new();
            for param in params {
                let token_value = Self::tokenize_parameter(&param.kind, &values[&param.name])?;
                tokens.push(Token { name: param.name.clone(), value: token_value});
            }

            Ok(tokens)
        } else {
            bail!(AbiErrorKind::WrongDataFormat(values.clone()))
        }
    }

    /// Tries to read tokens array from `Value`
    fn read_array(param: &ParamType, value: &Value) -> AbiResult<Vec<TokenValue>> {
        if let Value::Array(array) = value {
            let mut tokens = Vec::new();
            for value in array {
                tokens.push(Self::tokenize_parameter(param, value)?);
            }
            
            Ok(tokens)
        } else {
            bail!(AbiErrorKind::WrongDataFormat(value.clone()))
        }
    }

    /// Tries to parse a value as a vector of tokens of fixed size.
    fn tokenize_fixed_array(
        param: &ParamType,
        size: usize, value: &Value
    ) -> AbiResult<TokenValue> {
        let vec = Self::read_array(param, value)?;
        match vec.len() == size {
            true => Ok(TokenValue::FixedArray(vec)),
            false => bail!(AbiErrorKind::InvalidParameterLength(value.clone())),
        }
    }

    /// Tries to parse a value as a vector of tokens.
    fn tokenize_array(param: &ParamType, value: &Value) -> AbiResult<TokenValue> {
        let vec = Self::read_array(param, value)?;

        Ok(TokenValue::Array(vec))
    }

    /// Tries to parse a value as a bool.
    fn tokenize_bool(value: &Value) -> AbiResult<TokenValue> {
        match value {
            Value::Bool(value) => Ok(TokenValue::Bool(value.to_owned())),
            Value::String(string) => {
                match string.as_str() {
                    "true" => Ok(TokenValue::Bool(true)),
                    "false" => Ok(TokenValue::Bool(false)),
                    _ => bail!(AbiErrorKind::InvalidParameterValue(value.clone())),
                }
            }
            _ => bail!(AbiErrorKind::InvalidParameterValue(value.clone())),
        }
    }

    /// Tries to read integer number from `Value`
    fn read_int(value: &Value) -> AbiResult<BigInt> {
        if let Some(number) = value.as_i64() {
            Ok(BigInt::from(number))
        } else if let Some(string) = value.as_str() {
            let result = if string.starts_with("-0x") {
                BigInt::parse_bytes(&string.as_bytes()[3..], 16)
                .map(|number| -number)
            } else if string.starts_with("0x") {
                BigInt::parse_bytes(&string.as_bytes()[2..], 16)
            } else {
                BigInt::parse_bytes(string.as_bytes(), 10)
            };
            match result {
                Some(number) => Ok(number),
                None => bail!(AbiErrorKind::InvalidParameterValue(value.clone()))
            }
        } else {
            bail!(AbiErrorKind::WrongDataFormat(value.clone()))
        }
    }

    /// Tries to read integer number from `Value`
    fn read_uint(value: &Value) -> AbiResult<BigUint> {
        if let Some(number) = value.as_u64() {
            Ok(BigUint::from(number))
        } else if let Some(string) = value.as_str() {
            let result = if string.starts_with("0x") {
                BigUint::parse_bytes(&string.as_bytes()[2..], 16)
            } else {
                BigUint::parse_bytes(string.as_bytes(), 10)
            };
            match result {
                Some(number) => Ok(number),
                None => bail!(AbiErrorKind::InvalidParameterValue(value.clone()))
            }
        } else {
            bail!(AbiErrorKind::WrongDataFormat(value.clone()))
        }
    }

    /// Checks if given number can be fit into given bits count
    fn check_int_size(number: &BigInt, size: usize) -> bool {
        // `BigInt::bits` returns fewest bits necessary to express the number, not including
        // the sign and it works well for all values except `-2^n`. Such values can be encoded 
        // using `n` bits, but `bits` function returns `n` (and plus one bit for sign) so we 
        // have to explicitly check such situation by comparing bits sizes of given number 
        // and increased number
        if number.sign() == Sign::Minus && number.bits() != (number + BigInt::from(1)).bits() {
            number.bits() <= size
        } else {
            number.bits() < size
        }
    }

    /// Checks if given number can be fit into given bits count
    fn check_uint_size(number: &BigUint, size: usize) -> bool {
        number.bits() < size
    }

    /// Tries to parse a value as grams.
    fn tokenize_gram(value: &Value) -> AbiResult<TokenValue> {
        let number = Self::read_uint(value)?;

        if !Self::check_uint_size(&number, 120) {
            bail!(AbiErrorKind::InvalidParameterValue(value.clone()))
        } else {
            Ok(TokenValue::Gram(Grams::from(number)))
        }
    }

    /// Tries to parse a value as unsigned integer.
    fn tokenize_uint(size: usize, value: &Value) -> AbiResult<TokenValue> {
        let number = Self::read_uint(value)?;

        if !Self::check_uint_size(&number, size + 1) {
            bail!(AbiErrorKind::InvalidParameterValue(value.clone()))
        } else {
            Ok(TokenValue::Uint(Uint{number, size}))
        }
    }

    /// Tries to parse a value as signed integer.
    fn tokenize_int(size: usize, value: &Value) -> AbiResult<TokenValue> {
        let number = Self::read_int(value)?;

        if !Self::check_int_size(&number, size) {
            bail!(AbiErrorKind::InvalidParameterValue(value.clone()))
        } else {
            Ok(TokenValue::Int(Int{number, size}))
        }
    }

    fn tokenize_cell(value: &Value) -> AbiResult<TokenValue> {
        let string = value
            .as_str()
            .ok_or(AbiErrorKind::WrongDataFormat(value.clone()))?;
        let data = base64::decode(string)
            .map_err(|_| AbiErrorKind::InvalidParameterValue(value.clone()))?;
        let cell = deserialize_tree_of_cells(&mut Cursor::new(data))
            .map_err(|_| AbiErrorKind::InvalidParameterValue(value.clone()))?;
        Ok(TokenValue::Cell(cell))
    }

    fn tokenize_hashmap(key_type: &ParamType, value_type: &ParamType, map_value: &Value) -> AbiResult<TokenValue> {
        if let Value::Object(map) = map_value {
            let mut new_map = HashMap::<String, TokenValue>::new();
            for (key, value) in map.iter() {
                let value = Self::tokenize_parameter(value_type, value)?;
                new_map.insert(key.to_string(), value);
            }
            Ok(TokenValue::Map(key_type.clone(), new_map))
        } else {
            bail!(AbiErrorKind::WrongDataFormat(map_value.clone()))
        }
    }

    fn tokenize_bytes(value: &Value, size: Option<usize>) -> AbiResult<TokenValue> {
        let string = value
            .as_str()
            .ok_or(AbiErrorKind::WrongDataFormat(value.clone()))?;
        let mut data = hex::decode(string)
            .map_err(|_| AbiErrorKind::InvalidParameterValue(value.clone()))?;
        match size {
            Some(size) => if data.len() >= size {
                data.split_off(size);
                Ok(TokenValue::FixedBytes(data))
            } else {
                bail!(AbiErrorKind::InvalidParameterValue(value.clone()))
            }
            None => Ok(TokenValue::Bytes(data))
        }
    }

    /// Tries to parse a value as tuple.
    fn tokenize_tuple(params: &Vec<Param>, value: &Value) -> AbiResult<TokenValue> {
        let tokens = Self::tokenize_all(params, value)?;

        Ok(TokenValue::Tuple(tokens))
    }
}
