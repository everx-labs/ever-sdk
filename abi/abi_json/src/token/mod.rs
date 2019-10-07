//! TON ABI params.
use ton_abi_core::types::{
    Bitstring, Dint, Duint,
};
use types::int::Int;
use types::uint::Uint;
use {Param, ParamType};

use std::collections::BTreeMap;
use std::fmt;
use tvm::block::MsgAddress;

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
    /// dint: dynamic sized signed integer value.
    ///
    /// Encoded as Google Base 128 Varints put into cell data.
    Dint(Dint),
    /// duint: dynamic sized unsigned integer value.
    ///
    /// Encoded as Google Base 128 Varints put into cell data.
    Duint(Duint),
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
    /// bits<M>: static sized bits sequence.
    ///
    /// Encoding is equivalent to bool[M].
    Bits(Bitstring),
    /// bitstring: dynamic sized bits sequence.
    ///
    /// Encoding is equivalent to bool[].
    Bitstring(Bitstring),
    /// Dictionary of values
    ///
    Map(ParamType, BTreeMap<String, TokenValue>),
    /// MsgAddress
    ///
    Address(MsgAddress),
}

impl fmt::Display for TokenValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenValue::Uint(u) => write!(f, "{}", u.number),
            TokenValue::Int(u) => write!(f, "{}", u.number),
            TokenValue::Dint(d) => write!(f, "{}", d),
            TokenValue::Duint(d) => write!(f, "{}", d),
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
            TokenValue::Bits(b) => write!(f, "{}", b),
            TokenValue::Bitstring(b) => write!(f, "{}", b),
            TokenValue::Map(_key_type, map) => {
                let s = map
                    .iter()
                    .map(|ref t| format!("{}:{}", t.0, t.1))
                    .collect::<Vec<String>>()
                    .join(",");

                write!(f, "{{{}}}", s)
            }
            TokenValue::Address(a) => write!(f, "{}", serde_json::to_string(a).map_err(|_| fmt::Error)?),
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
            TokenValue::Dint(_) => *param_type == ParamType::Dint,
            TokenValue::Duint(_) => *param_type == ParamType::Duint,
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
            TokenValue::Bits(b) => {
                if let ParamType::Bits(size) = *param_type {
                    size == b.length_in_bits()
                } else {
                    false
                }
            }
            TokenValue::Bitstring(_) => *param_type == ParamType::Bitstring,
            TokenValue::Map(map_key_type, ref values) =>{
                if let ParamType::Map(ref key_type, ref value_type) = *param_type {
                    let key_type: &ParamType = key_type;
                    map_key_type == key_type || values.iter().all(|t| t.1.type_check(value_type))
                } else {
                    false
                }
            },
            TokenValue::Address(_) => *param_type == ParamType::Address,
        }
    }

    /// Returns `ParamType` the token value represents
    pub fn get_param_type(&self) -> ParamType {
        match self {
            TokenValue::Uint(uint) => ParamType::Uint(uint.size),
            TokenValue::Int(int) => ParamType::Int(int.size),
            TokenValue::Dint(_) => ParamType::Dint,
            TokenValue::Duint(_) => ParamType::Duint,
            TokenValue::Bool(_) => ParamType::Bool,
            TokenValue::Tuple(ref arr) => {
                ParamType::Tuple(arr.iter().map(|token| token.get_param()).collect())
            }
            TokenValue::Array(ref tokens) => ParamType::Array(Box::new(tokens[0].get_param_type())),
            TokenValue::FixedArray(ref tokens) => {
                ParamType::FixedArray(Box::new(tokens[0].get_param_type()), tokens.len())
            }
            TokenValue::Bits(b) => ParamType::Bits(b.length_in_bits()),
            TokenValue::Bitstring(_) => ParamType::Bitstring,
            TokenValue::Map(key_type, values) => ParamType::Map(Box::new(key_type.clone()), 
                Box::new(match values.iter().next() {
                    Some((_, value)) => value.get_param_type(),
                    None => ParamType::Unknown
            })),
            TokenValue::Address(_) => ParamType::Address,
        }
    }
}

impl Token {
    /// Check if all the types of the tokens match the given parameter types.
    pub fn types_check(tokens: &[Token], params: &[Param]) -> bool {
        params.len() == tokens.len() && {
            params.iter().zip(tokens).all(|(param, token)| {
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