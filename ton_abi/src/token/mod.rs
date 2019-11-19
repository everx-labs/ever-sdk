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

//! TON ABI params.
use int::{Int, Uint};
use {Param, ParamType};

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tvm::block::{Grams, MsgAddress};
use tvm::stack::CellData;

mod tokenizer;
mod detokenizer;
mod serialize;
mod deserialize;

pub use self::tokenizer::*;
pub use self::detokenizer::*;
pub use self::serialize::*;
pub use self::deserialize::*;

#[cfg(test)]
mod tests;

/// TON ABI params.
#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub name: String,
    pub value: TokenValue,
}

impl Token {
    pub fn new(name: &str, value: TokenValue) -> Self {
        Self { name: name.to_string(), value }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} : {}", self.name, self.value)
    }
}

/// TON ABI param values.
#[derive(Debug, PartialEq, Clone)]
pub enum TokenValue {
    /// uint<M>: unsigned integer type of M bits.
    ///
    /// Encoded as M bits of big-endian number representation put into cell data.
    Uint(Uint),
    /// int<M>: signed integer type of M bits.
    ///
    /// Encoded as M bits of big-endian number representation put into cell data.
    Int(Int),
    /// bool: boolean value.
    ///
    /// Encoded as one bit put into cell data.
    Bool(bool),
    /// Tuple: several values combinde into tuple.
    ///
    /// Encoded as all tuple elements encodings put into cell data one by one.
    Tuple(Vec<Token>),
    /// T[]: dynamic array of elements of the type T.
    ///
    /// Encoded as all array elements encodings put either to cell data or to separate cell.
    Array(Vec<TokenValue>),
    /// T[k]: dynamic array of elements of the type T.
    ///
    /// Encoded as all array elements encodings put either to cell data or to separate cell.
    FixedArray(Vec<TokenValue>),
    /// TVM Cell
    ///
    Cell(Arc<CellData>),
    /// Dictionary of values
    ///
    Map(ParamType, HashMap<String, TokenValue>),
    /// MsgAddress
    ///
    Address(MsgAddress),
    /// Raw byte array
    /// 
    /// Encoded as separate cells chain
    Bytes(Vec<u8>),
    /// Fixed sized raw byte array
    /// 
    /// Encoded as separate cells chain
    FixedBytes(Vec<u8>),
    /// Nanograms
    /// 
    Gram(Grams)
}

impl fmt::Display for TokenValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenValue::Uint(u) => write!(f, "{}", u.number),
            TokenValue::Int(u) => write!(f, "{}", u.number),
            TokenValue::Bool(b) => write!(f, "{}", b),
            TokenValue::Tuple(ref arr) => {
                let s = arr
                    .iter()
                    .map(|ref t| format!("{}", t))
                    .collect::<Vec<String>>()
                    .join(",");

                write!(f, "({})", s)
            }
            TokenValue::Array(ref arr) | TokenValue::FixedArray(ref arr) => {
                let s = arr
                    .iter()
                    .map(|ref t| format!("{}", t))
                    .collect::<Vec<String>>()
                    .join(",");

                write!(f, "[{}]", s)
            }
            TokenValue::Cell(c) => write!(f, "{:?}", c),
            TokenValue::Map(_key_type, map) => {
                let s = map
                    .iter()
                    .map(|ref t| format!("{}:{}", t.0, t.1))
                    .collect::<Vec<String>>()
                    .join(",");

                write!(f, "{{{}}}", s)
            }
            TokenValue::Address(a) => write!(f, "{}", a),
            TokenValue::Bytes(ref arr) | TokenValue::FixedBytes(ref arr) => write!(f, "{:?}", arr),
            TokenValue::Gram(g) => write!(f, "{}", g),
        }
    }
}

impl TokenValue {
    /// Check whether the type of the token matches the given parameter type.
    ///
    /// Numeric types (`Int` and `Uint`) type check if the size of the token
    /// type is of equal size with the provided parameter type.
    pub fn type_check(&self, param_type: &ParamType) -> bool {
        match self {
            TokenValue::Uint(uint) => *param_type == ParamType::Uint(uint.size),
            TokenValue::Int(int) => *param_type == ParamType::Int(int.size),
            TokenValue::Bool(_) => *param_type == ParamType::Bool,
            TokenValue::Tuple(ref arr) => {
                if let ParamType::Tuple(ref params) = *param_type {
                    Token::types_check(arr, &params)
                } else {
                    false
                }
            }
            TokenValue::Array(ref tokens) => {
                if let ParamType::Array(ref param_type) = *param_type {
                    tokens.iter().all(|t| t.type_check(param_type))
                } else {
                    false
                }
            }
            TokenValue::FixedArray(ref tokens) => {
                if let ParamType::FixedArray(ref param_type, size) = *param_type {
                    size == tokens.len() && tokens.iter().all(|t| t.type_check(param_type))
                } else {
                    false
                }
            }
            TokenValue::Cell(_) => *param_type == ParamType::Cell,
            TokenValue::Map(map_key_type, ref values) =>{
                if let ParamType::Map(ref key_type, ref value_type) = *param_type {
                    let key_type: &ParamType = key_type;
                    map_key_type == key_type && values.iter().all(|t| t.1.type_check(value_type))
                } else {
                    false
                }
            },
            TokenValue::Address(_) => *param_type == ParamType::Address,
            TokenValue::Bytes(_) => *param_type == ParamType::Bytes,
            TokenValue::FixedBytes(ref arr) => *param_type == ParamType::FixedBytes(arr.len()),
            TokenValue::Gram(_) => *param_type == ParamType::Gram,
        }
    }

    /// Returns `ParamType` the token value represents
    pub fn get_param_type(&self) -> ParamType {
        match self {
            TokenValue::Uint(uint) => ParamType::Uint(uint.size),
            TokenValue::Int(int) => ParamType::Int(int.size),
            TokenValue::Bool(_) => ParamType::Bool,
            TokenValue::Tuple(ref arr) => {
                ParamType::Tuple(arr.iter().map(|token| token.get_param()).collect())
            }
            TokenValue::Array(ref tokens) => ParamType::Array(Box::new(tokens[0].get_param_type())),
            TokenValue::FixedArray(ref tokens) => {
                ParamType::FixedArray(Box::new(tokens[0].get_param_type()), tokens.len())
            }
            TokenValue::Cell(_) => ParamType::Cell,
            TokenValue::Map(key_type, values) => ParamType::Map(Box::new(key_type.clone()), 
                Box::new(match values.iter().next() {
                    Some((_, value)) => value.get_param_type(),
                    None => ParamType::Unknown
            })),
            TokenValue::Address(_) => ParamType::Address,
            TokenValue::Bytes(_) => ParamType::Bytes,
            TokenValue::FixedBytes(ref arr) => ParamType::FixedBytes(arr.len()),
            TokenValue::Gram(_) => ParamType::Gram,
        }
    }
}

impl Token {
    /// Check if all the types of the tokens match the given parameter types.
    pub fn types_check(tokens: &[Token], params: &[Param]) -> bool {
        params.len() == tokens.len() && {
            params.iter().zip(tokens).all(|(param, token)| {
                // println!("{} {} {}", token.name, token.value, param.kind);
                token.value.type_check(&param.kind) && token.name == param.name
            })
        }
    }

    /// Rerturns `Param` the token represents
    pub fn get_param(&self) -> Param {
        Param {
            name: self.name.clone(),
            kind: self.value.get_param_type(),
        }
    }
}