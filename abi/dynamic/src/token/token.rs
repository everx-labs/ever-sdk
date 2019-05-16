//! TON ABI params.
use {ParamType, Param};
use types::int::Int;
use types::uint::Uint;
use abi_lib::types::{
    ABISerialized,
	ABIDeserialized,
    DeserializationError,
    get_next_bits_from_chain,
    bitstring_to_be_bytes,
	Dint,
	Duint,
	prepend_fixed_array,
	get_fixed_array_in_cell_size
};

use std::fmt;
use tvm::bitstring::Bitstring;
use tvm::stack::{BuilderData, SliceData};
use num_bigint::{BigInt, BigUint};

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
				let s = arr.iter()
					.map(|ref t| format!("{}", t))
					.collect::<Vec<String>>()
					.join(",");

				write!(f, "({})", s)
			},
			TokenValue::Array(ref arr) | TokenValue::FixedArray(ref arr) => {
				let s = arr.iter()
					.map(|ref t| format!("{}", t))
					.collect::<Vec<String>>()
					.join(",");

				write!(f, "[{}]", s)
			},
			TokenValue::Bits(b) => write!(f, "{}", b),
			TokenValue::Bitstring(b) => write!(f, "{}", b),
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
			TokenValue::Tuple(ref arr) =>
				if let ParamType::Tuple(ref params) = *param_type {
					Token::types_check(arr, &params)
				} else {
					false
				},
			TokenValue::Array(ref tokens) =>
				if let ParamType::Array(ref param_type) = *param_type {
					tokens.iter().all(|t| t.type_check(param_type))
				} else {
					false
				},
			TokenValue::FixedArray(ref tokens) =>
				if let ParamType::FixedArray(ref param_type, size) = *param_type {
					size == tokens.len() && tokens.iter().all(|t| t.type_check(param_type))
				} else {
					false
				},
			TokenValue::Bits(b) => 
				if let ParamType::Bits(size) = *param_type {
					size == b.length_in_bits()
				} else {
					false
				},
			TokenValue::Bitstring(_) => *param_type == ParamType::Bitstring,
		}
	}
}

impl Token {
	/// Check if all the types of the tokens match the given parameter types.
	pub fn types_check(tokens: &[Token], params: &[Param]) -> bool {
		params.len() == tokens.len() && {
			params.iter().zip(tokens).all(|(param, token)| {
				token.value.type_check(&param.kind) &&
				token.name == param.name
			})
		}
	}
}

impl ABISerialized for TokenValue {
	fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        match self {
			TokenValue::Uint(uint) => uint.prepend_to(destination),
			TokenValue::Int(int) => int.prepend_to(destination),
			TokenValue::Dint(dint) => dint.prepend_to(destination),
			TokenValue::Duint(duint) => duint.prepend_to(destination),
			TokenValue::Bool(b) => b.prepend_to(destination),
			TokenValue::Tuple(ref tokens) =>{
				let mut destination = destination;
				for token in tokens.iter().rev() {
					destination = token.value.prepend_to(destination);
				}
				destination
			},
			TokenValue::Array(ref tokens) => tokens.prepend_to(destination),
			TokenValue::FixedArray(ref tokens) => tokens.prepend_to(destination),
			TokenValue::Bits(b) => prepend_fixed_array(destination, &b.bits(0 .. b.length_in_bits()).data),
			TokenValue::Bitstring(bitstring) => bitstring.prepend_to(destination),
		}
    }

    fn get_in_cell_size(&self) -> usize {
        match self {
			TokenValue::Uint(uint) => uint.size,
			TokenValue::Int(int) => int.size,
			TokenValue::Dint(dint) => dint.get_in_cell_size(),
			TokenValue::Duint(duint) => duint.get_in_cell_size(),
			TokenValue::Bool(b) => 1,
			TokenValue::Tuple(ref tokens) =>{
				tokens.iter().fold(0usize, |size, token| size + token.value.get_in_cell_size())
			},
			TokenValue::Array(ref tokens) => tokens.get_in_cell_size(),
			TokenValue::FixedArray(ref tokens) => get_fixed_array_in_cell_size(&tokens),
			TokenValue::Bits(b) => get_fixed_array_in_cell_size(&b.bits(0 .. b.length_in_bits()).data),
			TokenValue::Bitstring(bitstring) => bitstring.get_in_cell_size(),
		}
    }
}

impl TokenValue {
	/// Deserializes value from `SliceData` to `TokenValue`
    pub fn read_from(param_type: &ParamType, cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
		match param_type {
			ParamType::Uint(size) => Self::read_uint(*size, cursor),
			ParamType::Int(size) => Self::read_int(*size, cursor),
			ParamType::Dint => {
				let (dint, cursor) = Dint::read_from(cursor)?;
				Ok((TokenValue::Dint(dint), cursor))
			},
			ParamType::Duint => {
				let (duint, cursor) = Duint::read_from(cursor)?;
				Ok((TokenValue::Duint(duint), cursor))
			},
			ParamType::Bool => {
				let (b, cursor) = bool::read_from(cursor)?;
				Ok((TokenValue::Bool(b), cursor))
			},
			ParamType::Tuple(tuple_params) => Self::read_tuple(tuple_params, cursor),
			ParamType::Array(param_type) => Self::read_array(&param_type, cursor),
			ParamType::FixedArray(param_type, size) => Self::read_fixed_array(&param_type, *size, cursor),
			ParamType::Bits(size) => Self::read_bits(*size, cursor),
			ParamType::Bitstring => {
				let (bitstring, cursor) = Bitstring::read_from(cursor)?;
				Ok((TokenValue::Bitstring(bitstring), cursor))
			},
		}
	}

	fn read_uint(size: usize, cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
        let (bitstring, cursor) = get_next_bits_from_chain(cursor, size)?;

        let vec = bitstring_to_be_bytes(bitstring, false);

        let result = Uint{
            number: BigUint::from_bytes_be(&vec),
            size: size
        };

        Ok((TokenValue::Uint(result), cursor))
	}

	fn read_int(size: usize, cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
        let (bitstring, cursor) = get_next_bits_from_chain(cursor, size)?;

        let vec = bitstring_to_be_bytes(bitstring, true);

        let result = Int{
            number: BigInt::from_signed_bytes_be(&vec),
            size: size
        };

        Ok((TokenValue::Int(result), cursor))
	}

	fn read_tuple(tuple_params: &[Param], cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
		let mut tokens = Vec::new();
		let mut cursor = cursor;
		for param in tuple_params {
			let (token_value, new_cursor) = TokenValue::read_from(&param.kind, cursor)?;
			tokens.push(Token { name: param.name.clone(), value: token_value });
			cursor = new_cursor;
		}
		Ok((TokenValue::Tuple(tokens), cursor))
	}

	fn read_bits(size: usize, cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
		let (token, cursor) = Self::read_fixed_array(&ParamType::Bool, size, cursor)?;

		if let TokenValue::FixedArray(array) = token {
			let bitstring = array
				.iter()
				.fold(
					Bitstring::new(),
					|mut bitstring, token| {
						if let TokenValue::Bool(b) = token {
							bitstring.append_bit_bool(*b);
							bitstring
						} else {
							panic!("Can't be here");
						}
					});

			Ok((TokenValue::Bits(bitstring), cursor))
		} else {
			panic!("Can't be here");
		}
	}

	fn read_array(param_type: &ParamType, cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
		let (flag, mut cursor) = <(bool,bool)>::read_from(cursor)?;

		match flag {
			(false, false) => {
				if cursor.remaining_references() == 0 {
					return Err(DeserializationError::with(cursor));
				}

				let mut array_cursor = cursor.checked_drain_reference().unwrap();
				let mut result = vec![];

				while array_cursor.remaining_references() != 0 || array_cursor.remaining_bits() != 0 {
					let (token, new_cursor) = Self::read_from(param_type, cursor)?;
					cursor = new_cursor;
					result.push(token);
				}

				Ok((TokenValue::Array(result), cursor))
			}
			(true, false) => {
                let (size, mut cursor) = <u8>::read_from(cursor)?;
                let mut result = vec![];

                for _ in 0..size {
                    let (token, new_cursor) = Self::read_from(param_type, cursor)?;
					cursor = new_cursor;
					result.push(token);
                }

                Ok((TokenValue::Array(result), cursor))
			}
			_ => Err(DeserializationError::with(cursor)),
		}
	}

	fn read_fixed_array(param_type: &ParamType, size: usize, cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
		let (flag, mut cursor) = <(bool,bool)>::read_from(cursor)?;

		match flag {
			(false, false) => {
				if cursor.remaining_references() == 0 {
					return Err(DeserializationError::with(cursor));
				}

				let mut array_cursor = cursor.checked_drain_reference().unwrap();
				let mut result = vec![];

				 for _ in 0..size {
					let (token, new_cursor) = Self::read_from(param_type, cursor)?;
					cursor = new_cursor;
					result.push(token);
				}

				if array_cursor.remaining_references() != 0 || array_cursor.remaining_bits() != 0 {
					return Err(DeserializationError::with(array_cursor));
				}

				Ok((TokenValue::Array(result), cursor))
			}
			(true, false) => {
                let mut result = vec![];

                for _ in 0..size {
                    let (token, new_cursor) = Self::read_from(param_type, cursor)?;
					cursor = new_cursor;
					result.push(token);
                }

                Ok((TokenValue::FixedArray(result), cursor))
			}
			_ => Err(DeserializationError::with(cursor)),
		}
	}
}


#[cfg(test)]
mod tests {
	use {TokenValue, Token, ParamType, Param, Uint, Int};
	use num_bigint::{BigInt, BigUint};
	use tvm::bitstring::Bitstring;

	#[test]
	fn test_type_check() {
		fn assert_type_check(tokens: &[Token], params: &[Param]) {
			assert!(Token::types_check(&tokens, params))
		}

		fn assert_not_type_check(tokens: &[Token], params: &[Param]) {
			assert!(!Token::types_check(&tokens, params))
		}

		let big_int = BigInt::from(123);
		let big_uint = BigUint::from(456u32);

		let tokens = vec![
			Token { 
				name: "a".to_owned(),
				value: TokenValue::Uint(Uint{number: big_uint.clone(), size: 32}) },
			Token { 
				name: "b".to_owned(),
				value: TokenValue::Int(Int{number: big_int.clone(), size: 64}) },
			Token { 
				name: "c".to_owned(),
				value: TokenValue::Dint(big_int.clone().into()) },
			Token { 
				name: "d".to_owned(),
				value: TokenValue::Duint(big_uint.clone().into()) },
			Token { 
				name: "e".to_owned(),
				value: TokenValue::Bool(false) },
			Token { 
				name: "f".to_owned(),
				value: TokenValue::Array(vec![TokenValue::Bool(false), TokenValue::Bool(true)]) },
			Token { 
				name: "g".to_owned(),
				value: TokenValue::FixedArray(vec![TokenValue::Dint(big_int.clone().into()), TokenValue::Dint(big_int.clone().into())]) },
			Token { 
				name: "h".to_owned(),
				value: TokenValue::Bits(Bitstring::create(vec![1, 2, 3], 15)) },
			Token { 
				name: "i".to_owned(),
				value: TokenValue::Bitstring(Bitstring::create(vec![1, 2, 3], 7)) },
			Token { 
				name: "j".to_owned(),
				value: TokenValue::Tuple(vec![
					Token { name: "a".to_owned(), value: TokenValue::Bool(true) },
					Token { name: "b".to_owned(), value: TokenValue::Duint(big_uint.clone().into()) } 
				]) },
		];

		let tuple_params = vec![	
			Param {name: "a".to_owned(), kind: ParamType::Bool},
			Param {name: "b".to_owned(), kind: ParamType::Duint},
		];

		let params = vec![
			Param {name: "a".to_owned(), kind: ParamType::Uint(32)},
			Param {name: "b".to_owned(), kind: ParamType::Int(64)},
			Param {name: "c".to_owned(), kind: ParamType::Dint},
			Param {name: "d".to_owned(), kind: ParamType::Duint},			
			Param {name: "e".to_owned(), kind: ParamType::Bool},
			Param {name: "f".to_owned(), kind: ParamType::Array(Box::new(ParamType::Bool))},
			Param {name: "g".to_owned(), kind: ParamType::FixedArray(Box::new(ParamType::Dint), 2)},
			Param {name: "h".to_owned(), kind: ParamType::Bits(15)},
			Param {name: "i".to_owned(), kind: ParamType::Bitstring},
			Param {name: "j".to_owned(), kind: ParamType::Tuple(tuple_params)},

		];

		assert_type_check(&tokens, &params);


		let mut tokens_wrong_type = tokens.clone();
		tokens_wrong_type[0] = Token { 
				name: "a".to_owned(),
				value: TokenValue::Bool(false) };
		assert_not_type_check(&tokens_wrong_type, &params);

		let mut tokens_wrong_int_size = tokens.clone();
		tokens_wrong_int_size[0] = Token { 
				name: "a".to_owned(),
				value: TokenValue::Uint(Uint{number: big_uint.clone(), size: 30}) };
		assert_not_type_check(&tokens_wrong_int_size, &params);

		let mut tokens_wrong_parameters_count = tokens.clone();
		tokens_wrong_parameters_count.pop();
		assert_not_type_check(&tokens_wrong_parameters_count, &params);

		let mut tokens_wrong_fixed_array_size = tokens.clone();
		tokens_wrong_fixed_array_size[6] = Token { 
				name: "g".to_owned(),
				value: TokenValue::FixedArray(vec![TokenValue::Dint(big_int.clone().into())]) };
		assert_not_type_check(&tokens_wrong_fixed_array_size, &params);

		let mut tokens_wrong_array_type = tokens.clone();
		tokens_wrong_array_type[5] = Token { 
				name: "f".to_owned(),
				value: TokenValue::Array(vec![TokenValue::Bool(false), TokenValue::Dint(big_int.clone().into())]) };
		assert_not_type_check(&tokens_wrong_array_type, &params);

		let mut tokens_wrong_tuple_type = tokens.clone();
		tokens_wrong_tuple_type[9] = Token { 
				name: "f".to_owned(),
				value: TokenValue::Tuple(vec![
					Token { name: "a".to_owned(), value: TokenValue::Int(Int{number: big_int.clone(), size: 16}) },
					Token { name: "b".to_owned(), value: TokenValue::Duint(big_uint.clone().into()) }])
		};
		assert_not_type_check(&tokens_wrong_tuple_type, &params);
	}
}
