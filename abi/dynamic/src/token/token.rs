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
	prepend_fixed_array
};

use std::fmt;
use tvm::bitstring::Bitstring;
use tvm::stack::{BuilderData, SliceData};
use num_bigint::{BigInt, BigUint};

/// TON ABI params.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
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
	Array(Vec<Token>),
	/// T[k]: dynamic array of elements of the type T.
	///
	/// Encoded as all array elements encodings put either to cell data or to separate cell.
	FixedArray(Vec<Token>),
	/// bits<M>: static sized bits sequence.
	///
	/// Encoding is equivalent to bool[M].
	Bits(Bitstring),
	/// bitstring: dynamic sized bits sequence.
	///
	/// Encoding is equivalent to bool[].
	Bitstring(Bitstring),
}

impl fmt::Display for Token {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Token::Uint(u) => write!(f, "{}", u.number),
			Token::Int(u) => write!(f, "{}", u.number),
			Token::Dint(d) => write!(f, "{}", d),
			Token::Duint(d) => write!(f, "{}", d),
			Token::Bool(b) => write!(f, "{}", b),
			Token::Tuple(ref arr) => {
				let s = arr.iter()
					.map(|ref t| format!("{}", t))
					.collect::<Vec<String>>()
					.join(",");

				write!(f, "({})", s)
			},
			Token::Array(ref arr) | Token::FixedArray(ref arr) => {
				let s = arr.iter()
					.map(|ref t| format!("{}", t))
					.collect::<Vec<String>>()
					.join(",");

				write!(f, "[{}]", s)
			},
			Token::Bits(b) => write!(f, "{}", b),
			Token::Bitstring(b) => write!(f, "{}", b),
		}
	}
}

impl Token {
	/// Check whether the type of the token matches the given parameter type.
	///
	/// Numeric types (`Int` and `Uint`) type check if the size of the token
	/// type is of equal size with the provided parameter type.
	pub fn type_check(&self, param_type: &ParamType) -> bool {
		match self {
			Token::Uint(uint) => *param_type == ParamType::Uint(uint.size),
			Token::Int(int) => *param_type == ParamType::Int(int.size),
			Token::Dint(_) => *param_type == ParamType::Dint,
			Token::Duint(_) => *param_type == ParamType::Duint,
			Token::Bool(_) => *param_type == ParamType::Bool,
			Token::Tuple(ref arr) =>
				if let ParamType::Tuple(ref params) = *param_type {
					Self::types_check(arr, &params)
				} else {
					false
				},
			Token::Array(ref tokens) =>
				if let ParamType::Array(ref param_type) = *param_type {
					tokens.iter().all(|t| t.type_check(param_type))
				} else {
					false
				},
			Token::FixedArray(ref tokens) =>
				if let ParamType::FixedArray(ref param_type, size) = *param_type {
					size == tokens.len() && tokens.iter().all(|t| t.type_check(param_type))
				} else {
					false
				},
			Token::Bits(b) => 
				if let ParamType::Bits(size) = *param_type {
					size == b.length_in_bits()
				} else {
					false
				},
			Token::Bitstring(_) => *param_type == ParamType::Bitstring,
		}
	}

	/// Check if all the types of the tokens match the given parameter types.
	pub fn types_check(tokens: &[Token], params: &[Param]) -> bool {
		params.len() == tokens.len() && {
			params.iter().zip(tokens).all(|(param, token)| {
				token.type_check(&param.kind)
			})
		}
	}
}

impl ABISerialized for Token {
	fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        match self {
			Token::Uint(uint) => uint.prepend_to(destination),
			Token::Int(int) => int.prepend_to(destination),
			Token::Dint(dint) => dint.prepend_to(destination),
			Token::Duint(duint) => duint.prepend_to(destination),
			Token::Bool(b) => b.prepend_to(destination),
			Token::Tuple(ref tokens) =>{
				let mut destination = destination;
				for token in tokens.iter().rev() {
					destination = token.prepend_to(destination);
				}
				destination
			},
			Token::Array(ref tokens) => tokens.prepend_to(destination),
			Token::FixedArray(ref tokens) => tokens.prepend_to(destination),
			Token::Bits(b) => prepend_fixed_array(destination, &b.bits(0 .. b.length_in_bits()).data),
			Token::Bitstring(bitstring) => bitstring.prepend_to(destination),
		}
    }

    fn get_in_cell_size(&self) -> usize {
        1
    }
}

impl Token {
	/// Deserializes value from `SliceData` to `Token`
    pub fn read_from(param_type: &ParamType, cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
		match param_type {
			ParamType::Uint(size) => Self::read_uint(*size, cursor),
			ParamType::Int(size) => Self::read_int(*size, cursor),
			ParamType::Dint => {
				let (dint, cursor) = Dint::read_from(cursor)?;
				Ok((Token::Dint(dint), cursor))
			},
			ParamType::Duint => {
				let (duint, cursor) = Duint::read_from(cursor)?;
				Ok((Token::Duint(duint), cursor))
			},
			ParamType::Bool => {
				let (b, cursor) = bool::read_from(cursor)?;
				Ok((Token::Bool(b), cursor))
			},
			ParamType::Tuple(tuple_params) => Self::read_tuple(tuple_params, cursor),
			ParamType::Array(param_type) => Self::read_array(&param_type, cursor),
			ParamType::FixedArray(param_type, size) => Self::read_fixed_array(&param_type, *size, cursor),
			ParamType::Bits(size) => Self::read_bits(*size, cursor),
			ParamType::Bitstring => {
				let (bitstring, cursor) = Bitstring::read_from(cursor)?;
				Ok((Token::Bitstring(bitstring), cursor))
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

        Ok((Token::Uint(result), cursor))
	}

	fn read_int(size: usize, cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
        let (bitstring, cursor) = get_next_bits_from_chain(cursor, size)?;

        let vec = bitstring_to_be_bytes(bitstring, true);

        let result = Int{
            number: BigInt::from_signed_bytes_be(&vec),
            size: size
        };

        Ok((Token::Int(result), cursor))
	}

	fn read_tuple(tuple_params: &[Param], cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
		let mut tokens = Vec::new();
		let mut cursor = cursor;
		for param in tuple_params {
			let (token, new_cursor) = Token::read_from(&param.kind, cursor)?;
			tokens.push(token);
			cursor = new_cursor;
		}
		Ok((Token::Tuple(tokens), cursor))
	}

	fn read_bits(size: usize, cursor: SliceData) -> Result<(Self, SliceData), DeserializationError> {
		let (token, cursor) = Self::read_fixed_array(&ParamType::Bool, size, cursor)?;

		if let Token::FixedArray(array) = token {
			let bitstring = array
				.iter()
				.fold(
					Bitstring::new(),
					|mut bitstring, token| {
						if let Token::Bool(b) = token {
							bitstring.append_bit_bool(*b);
							bitstring
						} else {
							panic!("Can't be here");
						}
					});

			Ok((Token::Bits(bitstring), cursor))
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

				Ok((Token::Array(result), cursor))
			}
			(true, false) => {
                let (size, mut cursor) = <u8>::read_from(cursor)?;
                let mut result = vec![];

                for _ in 0..size {
                    let (token, new_cursor) = Self::read_from(param_type, cursor)?;
					cursor = new_cursor;
					result.push(token);
                }

                Ok((Token::Array(result), cursor))
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

				Ok((Token::Array(result), cursor))
			}
			(true, false) => {
                let mut result = vec![];

                for _ in 0..size {
                    let (token, new_cursor) = Self::read_from(param_type, cursor)?;
					cursor = new_cursor;
					result.push(token);
                }

                Ok((Token::FixedArray(result), cursor))
			}
			_ => Err(DeserializationError::with(cursor)),
		}
	}
}
/*
impl serde::Serialize for Token {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
			Token::Uint(uint) => uint.prepend_to(destination),
			Token::Int(int) => int.prepend_to(destination),
			Token::Dint(dint) => dint.prepend_to(destination),
			Token::Duint(duint) => duint.prepend_to(destination),
			Token::Bool(b) => b.prepend_to(destination),
			Token::Tuple(ref tokens) =>{
				let mut destination = destination;
				for token in tokens.iter().rev() {
					destination = token.prepend_to(destination);
				}
				destination
			},
			Token::Array(ref tokens) => tokens.prepend_to(destination),
			Token::FixedArray(ref tokens) => tokens.prepend_to(destination),
			Token::Bits(b) => prepend_fixed_array(destination, &b.bits(0 .. b.length_in_bits()).data),
			Token::Bitstring(bitstring) => bitstring.prepend_to(destination),
		}
    }
}*/


#[cfg(test)]
mod tests {
	use {Token, ParamType};

	#[test]
	fn test_type_check() {
		fn assert_type_check(tokens: Vec<Token>, param_types: Vec<ParamType>) {
			assert!(Token::types_check(&tokens, &param_types))
		}

		fn assert_not_type_check(tokens: Vec<Token>, param_types: Vec<ParamType>) {
			assert!(!Token::types_check(&tokens, &param_types))
		}

		assert_type_check(vec![Token::Uint(0.into()), Token::Bool(false)], vec![ParamType::Uint(256), ParamType::Bool]);
		assert_type_check(vec![Token::Uint(0.into()), Token::Bool(false)], vec![ParamType::Uint(32), ParamType::Bool]);

		assert_not_type_check(vec![Token::Uint(0.into())], vec![ParamType::Uint(32), ParamType::Bool]);
		assert_not_type_check(vec![Token::Uint(0.into()), Token::Bool(false)], vec![ParamType::Uint(32)]);
		assert_not_type_check(vec![Token::Bool(false), Token::Uint(0.into())], vec![ParamType::Uint(32), ParamType::Bool]);

		assert_type_check(vec![Token::FixedBytes(vec![0, 0, 0, 0])], vec![ParamType::FixedBytes(4)]);
		assert_type_check(vec![Token::FixedBytes(vec![0, 0, 0])], vec![ParamType::FixedBytes(4)]);
		assert_not_type_check(vec![Token::FixedBytes(vec![0, 0, 0, 0])], vec![ParamType::FixedBytes(3)]);

		assert_type_check(vec![Token::Array(vec![Token::Bool(false), Token::Bool(true)])], vec![ParamType::Array(Box::new(ParamType::Bool))]);
		assert_not_type_check(vec![Token::Array(vec![Token::Bool(false), Token::Uint(0.into())])], vec![ParamType::Array(Box::new(ParamType::Bool))]);
		assert_not_type_check(vec![Token::Array(vec![Token::Bool(false), Token::Bool(true)])], vec![ParamType::Array(Box::new(ParamType::Address))]);

		assert_type_check(vec![Token::FixedArray(vec![Token::Bool(false), Token::Bool(true)])], vec![ParamType::FixedArray(Box::new(ParamType::Bool), 2)]);
		assert_not_type_check(vec![Token::FixedArray(vec![Token::Bool(false), Token::Bool(true)])], vec![ParamType::FixedArray(Box::new(ParamType::Bool), 3)]);
		assert_not_type_check(vec![Token::FixedArray(vec![Token::Bool(false), Token::Uint(0.into())])], vec![ParamType::FixedArray(Box::new(ParamType::Bool), 2)]);
		assert_not_type_check(vec![Token::FixedArray(vec![Token::Bool(false), Token::Bool(true)])], vec![ParamType::FixedArray(Box::new(ParamType::Address), 2)]);
	}
}
