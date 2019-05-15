use std::collections::HashMap;
use serde::ser::{Serialize, Serializer, SerializeMap};
use {ParamType, Param, Uint, Int, Token};
use num_bigint::{Sign, BigInt, BigUint};
use tvm::bitstring::{Bitstring, Bit};

#[derive(Debug)]
pub enum DetokenizeError {
    WrongParametersCount,
    SerdeError(serde_json::Error),
    WrongParameterType,
}

pub struct Detokenizer;

impl Detokenizer {
    pub fn detokenize(params: &[Param], tokens: &[Token]) -> Result<String, DetokenizeError> {
        //println!("Params len = {}, tokens len = {}", params.len(), tokens.len());

        if params.len() != tokens.len() {
            return Err(DetokenizeError::WrongParametersCount);
        }

        if !Token::types_check(tokens, params) {
            return Err(DetokenizeError::WrongParameterType);
        }

        let tuples_vec = params
            .iter()
            .zip(tokens)
            .collect();

        serde_json::to_string(&FunctionParams{params: tuples_vec}).map_err(|err| DetokenizeError::SerdeError(err))
    }
}

pub struct FunctionParams<'a> {
    params: Vec<(&'a Param, &'a Token)>,
}

impl<'a> Serialize for FunctionParams<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.params.len()))?;

        for (param, token) in &self.params {
            if let ParamType::Tuple(ref tuple_params) = param.kind {
                if let Token::Tuple(ref tuple_tokens) = token {
                    let tuples_vec = tuple_params
                        .iter()
                        .zip(tuple_tokens)
                        .collect();

                    map.serialize_entry(param, &FunctionParams{params: tuples_vec})?;
                }
            } else {
                map.serialize_entry(param, token)?;
            }
        }
        map.end()
    }
}

impl Token {
	pub fn detokenize_big_int<S>(number: &BigInt, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let int_str = "0x".to_owned() + &number.to_str_radix(16);

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
}

impl Serialize for Token {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
		match self {
			Token::Uint(uint) => Token::detokenize_big_uint(&uint.number, serializer),
			Token::Int(int) => Token::detokenize_big_int(&int.number, serializer),
			Token::Dint(dint) => Token::detokenize_big_int(&dint, serializer),
			Token::Duint(duint) => Token::detokenize_big_uint(&duint, serializer),
			Token::Bool(b) => serializer.serialize_bool(b.clone()),
			Token::Tuple(_) => panic!("Shouldn't be here! Tuple should be serialized as `map`"),
			Token::Array(ref tokens) => tokens.serialize(serializer),
			Token::FixedArray(ref tokens) => tokens.serialize(serializer),
			Token::Bits(bitstring) => Token::detokenize_bitstring(&bitstring, serializer),
			Token::Bitstring(bitstring) => Token::detokenize_bitstring(&bitstring, serializer),
		}
    }
}