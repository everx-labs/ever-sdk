//! ABI param and parsing for it.
use {ParamType, Param, Uint, Int, Token, TokenValue};
use serde_json::Value;
use num_bigint::{Sign, BigInt};
use tvm::stack::{BuilderData, IBitstring};

/// Returning errors during deserialization
#[derive(Debug)]
pub enum TokenizeError {
    WrongDataFormat(Value),
    WrongParametersCount,
    InvalidParameterLength(Value),
    InvalidParameterValue(Value),
    NotImplemented,
}

/// This struct should be used to parse string values as tokens.
pub struct Tokenizer;

impl Tokenizer {
    /// Tries to parse a JSON value as a token of given type.
    fn tokenize_parameter(param: &ParamType, value: &Value) -> Result<TokenValue, TokenizeError> {
        match param {
            ParamType::Uint(size) => Self::tokenize_uint(*size, value),
            ParamType::Int(size) => Self::tokenize_int(*size, value),
            ParamType::Dint => Self::tokenize_dint(value),
            ParamType::Duint => Self::tokenize_duint(value),
            ParamType::Bool => Self::tokenize_bool(value),
            ParamType::Tuple(tuple_params) => Self::tokenize_tuple(tuple_params, value),
            ParamType::Array(param_type) => Self::tokenize_array(&param_type, value),
            ParamType::FixedArray(param_type, size) => Self::tokenize_fixed_array(&param_type, *size, value),
            ParamType::Bits(size) => Self::tokenize_bits(*size, value),
            ParamType::Bitstring => Self::tokenize_bitstring(value),
        }
    }

    /// Tries to parse parameters from JSON values to tokens.
    pub fn tokenize_all(params: &[Param], values: &Value) -> Result<Vec<Token>, TokenizeError> {
        if let Value::Object(map) = values {
            if map.len() != params.len() {
                return Err(TokenizeError::WrongParametersCount);
            }

            let mut tokens = Vec::new();
            for param in params {
                let token_value = Self::tokenize_parameter(&param.kind, &values[&param.name])?;
                tokens.push(Token { name: param.name.clone(), value: token_value});
            }

            Ok(tokens)
        } else {
            Err(TokenizeError::WrongDataFormat(values.clone()))
        }
    }

    /// Tries to read tokens array from `Value`
    fn read_array(param: &ParamType, value: &Value) -> Result<Vec<TokenValue>, TokenizeError> {
        if let Value::Array(array) = value {
            let mut tokens = Vec::new();
            for value in array {
                tokens.push(Self::tokenize_parameter(param, value)?);
            }
            
            Ok(tokens)
        } else {
            Err(TokenizeError::WrongDataFormat(value.clone()))
        }
    }

    /// Tries to parse a value as a vector of tokens of fixed size.
    fn tokenize_fixed_array(
        param: &ParamType,
        size: usize, value: &Value
    ) -> Result<TokenValue, TokenizeError> {
        let vec = Self::read_array(param, value)?;
        match vec.len() == size {
            true => Ok(TokenValue::FixedArray(vec)),
            false => Err(TokenizeError::InvalidParameterLength(value.clone())),
        }
    }

    /// Tries to parse a value as a vector of tokens.
    fn tokenize_array(param: &ParamType, value: &Value) -> Result<TokenValue, TokenizeError> {
        let vec = Self::read_array(param, value)?;

        Ok(TokenValue::Array(vec))
    }

    /// Tries to parse a value as a bool.
    fn tokenize_bool(value: &Value) -> Result<TokenValue, TokenizeError> {
        match value {
            Value::Bool(value) => Ok(TokenValue::Bool(value.to_owned())),
            Value::String(string) => {
                match string.as_str() {
                    "true" => Ok(TokenValue::Bool(true)),
                    "false" => Ok(TokenValue::Bool(false)),
                    _ => Err(TokenizeError::InvalidParameterValue(value.clone())),
                }
            }
            _ => Err(TokenizeError::InvalidParameterValue(value.clone())),
        }
    }

    /// Tries to read integer number from `Value`
    fn read_int(value: &Value) -> Result<BigInt, TokenizeError> {
        if value.is_i64() {
            let number = value.as_i64().unwrap();

            Ok(BigInt::from(number))
        } else if value.is_string() {
            let mut string = value.as_str().unwrap().to_owned();

            let radix = if string.starts_with("-") {
                if string.starts_with("-0x") {
                    string.replace_range(1..3, "");
                    16
                } else {
                    10
                }
            } else {
                if string.starts_with("0x") {
                    string.replace_range(0..2, "");
                    16
                } else {
                    10
                }
            };

            let number = BigInt::parse_bytes(string.as_bytes(), radix)
                            .ok_or(TokenizeError::InvalidParameterValue(value.clone()))?;

            Ok(number)
        } else {
            Err(TokenizeError::WrongDataFormat(value.clone()))
        }
    }

    /// Checks if given number can be fit into given bits count
    fn check_int_size(number: &BigInt, size: usize) -> bool {
        // `BigInt::bits` returns fewest bits necessary to express the number, not including
        // the sign and it works well for all values except `-2^n`. Such values can be encoded 
        // using `n` bits, but `bits` function returns `n` (and plus one bit for sign) so we 
        // have to explicitly check such situation by comparing bits sizes of given number 
        // and increased number
        if    number.sign() == Sign::Minus &&
            number.bits() != (number + BigInt::from(1)).bits()
        { 
            number.bits() <= size
        } else {
            number.bits() < size
        }
    }

    /// Tries to parse a value as unsigned integer.
    fn tokenize_uint(size: usize, value: &Value) -> Result<TokenValue, TokenizeError> {
        let big_int = Self::read_int(value)?;

        let number = big_int.to_biguint().ok_or(TokenizeError::InvalidParameterValue(value.clone()))?;

        if !Self::check_int_size(&big_int, size + 1) {
            Err(TokenizeError::InvalidParameterValue(value.clone()))
        } else {
            Ok(TokenValue::Uint(Uint{number, size}))
        }
    }

    /// Tries to parse a value as signed integer.
    fn tokenize_int(size: usize, value: &Value) -> Result<TokenValue, TokenizeError> {
        let number = Self::read_int(value)?;

        if !Self::check_int_size(&number, size) {
            Err(TokenizeError::InvalidParameterValue(value.clone()))
        } else {
            Ok(TokenValue::Int(Int{number, size}))
        }
    }

    /// Tries to parse a value as a dynamic int.
    fn tokenize_dint(value: &Value) -> Result<TokenValue, TokenizeError> {
        let big_int = Self::read_int(value)?;

        Ok(TokenValue::Dint(big_int))
    }

    /// Tries to parse a value as a dynamic insigned int.
    fn tokenize_duint(value: &Value) -> Result<TokenValue, TokenizeError> {
        let big_int = Self::read_int(value)?;

        let big_uint = big_int
            .to_biguint()
            .ok_or(TokenizeError::InvalidParameterValue(value.clone()))?;

        Ok(TokenValue::Duint(big_uint))
    }

    /// Tries to read bitstring from `Value`.
    fn read_bitstring(value: &Value) -> Result<BuilderData, TokenizeError> {
        let mut string = value
            .as_str()
            .ok_or(TokenizeError::WrongDataFormat(value.clone()))?
            .to_owned();

        // hexademical representation
        let bitstring = if string.starts_with("x") {
            // trim additional symbols
            let square_brackets: &[_] = &['{', '}'];
            string = string.trim_start_matches("x").trim_matches(square_brackets).to_owned();

            // if bitstring length is not divisible by 8 then it is ended by `completion tag`
            // (see TON Blockchain spec)
            if string.ends_with("_") {
                // Pad bitstring with zeros to parse as normal hex-string. It will be trimmed 
                // using `completion tag`
                let len = string.len(); 
                string.replace_range(len - 1 .. len, "0");

                if string.len() % 2 != 0 {
                    string.push('0');
                }
            } else {
                // add `completion tag`
                string += "80";
            }

            let vec = hex::decode(string)
                .map_err(|_| TokenizeError::InvalidParameterValue(value.clone()))?;

            BuilderData::with_bitstring(vec)
        } else { // bits representation
            let mut bitstring = BuilderData::new();

            for bit in string.chars() {
                match bit {
                    '0' => bitstring.append_bit_zero().unwrap(),
                    '1' => bitstring.append_bit_one().unwrap(),
                    _ => return Err(TokenizeError::InvalidParameterValue(value.clone()))
                };
            }

            bitstring
        };

        Ok(bitstring)
    }

    /// Tries to parse a value as bitstring.
    fn tokenize_bitstring(value: &Value) -> Result<TokenValue, TokenizeError> {
        Self::read_bitstring(value).map(|bitstring| TokenValue::Bitstring(bitstring))
    }

    /// Tries to parse a value as fixed sized bits sequence.
    fn tokenize_bits(size: usize, value: &Value) -> Result<TokenValue, TokenizeError> {
        let bitstring = Self::read_bitstring(value)?;

        if bitstring.length_in_bits() != size {
            Err(TokenizeError::InvalidParameterLength(value.clone()))
        } else {
            Ok(TokenValue::Bits(bitstring))
        }
    }
    
    /// Tries to parse a value as tuple.
    fn tokenize_tuple(params: &Vec<Param>, value: &Value) -> Result<TokenValue, TokenizeError> {
        let tokens = Self::tokenize_all(params, value)?;

        Ok(TokenValue::Tuple(tokens))
    }
}
