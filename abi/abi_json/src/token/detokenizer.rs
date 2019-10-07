use serde::ser::{Serialize, Serializer, SerializeMap};
use std::collections::BTreeMap;
use {Param, ParamType, Token, TokenValue};
use num_bigint::{BigInt, BigUint};
use ton_abi_core::types::Bitstring;
use crate::error::*;

pub struct Detokenizer;

impl Detokenizer {
    pub fn detokenize(params: &[Param], tokens: &[Token]) -> AbiResult<String> {
        //println!("Params len = {}, tokens len = {}", params.len(), tokens.len());

        if params.len() != tokens.len() {
            bail!(AbiErrorKind::WrongParametersCount(params.len(), tokens.len()));
        }

        if !Token::types_check(tokens, params) {
             bail!(AbiErrorKind::WrongParameterType);
        }

        Ok(serde_json::to_string(&FunctionParams{params: tokens})?)
    }
}

pub struct FunctionParams<'a> {
    params: &'a [Token],
}

impl<'a> Serialize for FunctionParams<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.params.len()))?;

        for token in self.params {
                map.serialize_entry(&token.name, &token.value)?;
            }

        map.end()
    }
}

impl Token {
    pub fn detokenize_big_int<S>(number: &BigInt, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut int_str = number.to_str_radix(16);
        
        if int_str.starts_with("-") {
            int_str.insert_str(1, "0x");
        } else {
            int_str.insert_str(0, "0x");
        };

        serializer.serialize_str(&int_str)
    }

    pub fn detokenize_big_uint<S>(number: &BigUint, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let uint_str = "0x".to_owned() + &number.to_str_radix(16);

        serializer.serialize_str(&uint_str)
    }

    pub fn detokenize_bitstring<S>(bitstring: &Bitstring, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut vec = Vec::new();
        bitstring.into_bitstring_with_completion_tag(&mut vec);

        let set_tag = if vec[vec.len() - 1] == 0x80 {
            vec.pop();
            false
        } else {
            true
        };

        let mut string = "x".to_owned() + &hex::encode(vec);
        if set_tag {
            string += "_"
        }

        serializer.serialize_str(&string)
    }

    pub fn detokenize_hashmap<S>(_key_type: &ParamType, values: &BTreeMap<String, TokenValue>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(values.len()))?;
        for (k, v) in values {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl Serialize for TokenValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TokenValue::Uint(uint) => Token::detokenize_big_uint(&uint.number, serializer),
            TokenValue::Int(int) => Token::detokenize_big_int(&int.number, serializer),
            TokenValue::Dint(dint) => Token::detokenize_big_int(&dint, serializer),
            TokenValue::Duint(duint) => Token::detokenize_big_uint(&duint, serializer),
            TokenValue::Bool(b) => serializer.serialize_bool(b.clone()),
            TokenValue::Tuple(tokens) => {
                FunctionParams {params: tokens}.serialize(serializer)
            },
            TokenValue::Array(ref tokens) => tokens.serialize(serializer),
            TokenValue::FixedArray(ref tokens) => tokens.serialize(serializer),
            TokenValue::Bits(ref bitstring) => Token::detokenize_bitstring(bitstring, serializer),
            TokenValue::Bitstring(ref bitstring) => Token::detokenize_bitstring(bitstring, serializer),
            TokenValue::Map(key_type, ref map) => Token::detokenize_hashmap(key_type, map, serializer),
            TokenValue::Address(ref address) => address.serialize(serializer),
        }
    }
}