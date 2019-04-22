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

/// This trait should be used to parse string values as tokens.
pub struct Detokenizer;

impl Detokenizer {
    pub fn detokenize(params: &[Param], tokens: &[Token]) -> Result<String, DetokenizeError> {
        println!("Params len = {}, tokens len = {}", params.len(), tokens.len());

        if params.len() != tokens.len() {
            return Err(DetokenizeError::WrongParametersCount);
        }

        if !Token::types_check(tokens, params) {
            return Err(DetokenizeError::WrongParameterType);
        }

        let tokens_map = params
            .iter()
            .zip(tokens)
            .fold(HashMap::new(), |mut map, (param, token)| {
                map.insert(param.clone(), token.clone());
                map
            });

        serde_json::to_string(&FunctionParams{params: tokens_map}).map_err(|err| DetokenizeError::SerdeError(err))
    }
}

pub struct FunctionParams {
    params: HashMap<Param, Token>,
}

impl Serialize for FunctionParams {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.params.len()))?;

        for (param, token) in &self.params {
            if let ParamType::Tuple(ref tuple_params) = param.kind {
                if let Token::Tuple(ref tuple_tokens) = token {
                    let tokens_map = tuple_params
                        .iter()
                        .zip(tuple_tokens)
                        .fold(HashMap::new(), |mut map, (param, token)| {
                            map.insert(param.clone(), token.clone());
                            map
                        });

                    map.serialize_entry(param, &tokens_map)?;
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

        // TODO: only multiple of 8 sizes are supported now
        assert_eq!(vec.pop(), Some(0x80));

        let string = "x".to_owned() + &hex::encode(vec);

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