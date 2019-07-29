use serde::ser::{Serialize, Serializer, SerializeMap};
use {Param, Token, TokenValue};
use num_bigint::{BigInt, BigUint};
use tvm::stack::BuilderData;

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

        serde_json::to_string(&FunctionParams{params: tokens})
            .map_err(|err| DetokenizeError::SerdeError(err))
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

    pub fn detokenize_bitstring<S>(
        bitstring: &BuilderData, 
        serializer: S
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut vec = bitstring.cell().data().to_vec();
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
            TokenValue::Bits(bitstring) => Token::detokenize_bitstring(&bitstring, serializer),
            TokenValue::Bitstring(bitstring) => Token::detokenize_bitstring(&bitstring, serializer),
        }
    }
}