//! Contract function call builder.

use sha2::{Digest, Sha256, Sha512};
use {Param, Token};
use ed25519_dalek::*;
use tvm::stack::{BuilderData, SliceData};
use tvm::bitstring::Bitstring;
use tvm::cells_serialization::BagOfCells;
use abi_lib::types::prepend_data_to_chain;
use abi_lib::types::{ABISerialized, DeserializationError as InnerTypeDeserializationError};

pub const ABI_VERSION: u8 = 0;

/// Contract function specification.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Function {
	/// Function name.
	pub name: String,
	/// Function input.
	#[serde(default)]
	pub inputs: Vec<Param>,
	/// Function output.
	#[serde(default)]
	pub outputs: Vec<Param>,
	/// Signed function.
	#[serde(default)]
	pub signed: bool,
}

#[derive(Debug)]
pub enum SerializationError {
	WrongParameterType,
	KeyPairNeeded,
}

#[derive(Debug)]
pub enum DeserializationError {
	TypeDeserializationError(InnerTypeDeserializationError),
    IncompleteDeserializationError,
}

impl Function {
	/// Returns all input params of given function.
	pub fn input_params(&self) -> Vec<Param> {
		self.inputs.iter()
			.map(|p| p.clone())
			.collect()
	}

	/// Returns all output params of given function.
	pub fn output_params(&self) -> Vec<Param> {
		self.outputs.iter()
			.map(|p| p.clone())
			.collect()
	}

	/// Retruns ABI function signature
	pub fn get_function_signature(&self) -> String {
		let input_types = self.inputs.iter()
			.map(|param| param.kind.type_signature())
			.collect::<Vec<String>>()
			.join(",");

		let output_types = self.outputs.iter()
			.map(|param| param.kind.type_signature())
			.collect::<Vec<String>>()
			.join(",");

		format!("{}({})({})", self.name, input_types, output_types)
	}

	/// Computes function ID for contract function
    pub fn get_function_id(&self) -> [u8; 4] {
		let signature = self.get_function_signature();

        //println!("{}", signature);

        // Sha256 hash of signature
        let mut hasher = Sha256::new();

        hasher.input(&signature.into_bytes()[..]);

        let function_hash = hasher.result();

        let mut bytes = [0; 4];
        bytes.copy_from_slice(&function_hash[..4]);
        //println!("{:X?}", bytes);
        bytes
    }

	/// Parses the ABI function output to list of tokens.
	pub fn decode_output(&self, data: SliceData) -> Result<Vec<Token>, DeserializationError> {
		let params = self.output_params();

		let mut tokens = vec![];
		let mut cursor = data;

		for param in params {
			let (token, new_cursor) = Token::read_from(&param.kind, cursor)
										.map_err(|err| DeserializationError::TypeDeserializationError(err))?;
			cursor = new_cursor;
			tokens.push(token);
		}

		if cursor.remaining_references() != 0 || cursor.remaining_bits() != 0 {
            Err(DeserializationError::IncompleteDeserializationError)
        } else {
			Ok(tokens)
		}
	}

		
	/// Parses the ABI function output to list of tokens.
	pub fn decode_input(&self, data: SliceData) -> Result<Vec<Token>, DeserializationError> {
		use ParamType;
		let mut params = self.input_params();

		params.insert(0, Param{kind: ParamType::Int(32), name: "a".to_owned()});
		params.insert(0, Param{kind: ParamType::Uint(8), name: "b".to_owned()});

		let mut tokens = vec![];
		let mut cursor = data;

		if self.signed {
			cursor.drain_reference();
		}

		for param in params {
			let (token, new_cursor) = Token::read_from(&param.kind, cursor)
										.map_err(|err| DeserializationError::TypeDeserializationError(err))?;
			println!("{}", token);
			cursor = new_cursor;
			tokens.push(token);
		}

		if cursor.remaining_references() != 0 || cursor.remaining_bits() != 0 {
            Err(DeserializationError::IncompleteDeserializationError)
        } else {
			tokens.remove(0);
			tokens.remove(0);

			Ok(tokens)
		}
	}

    /// Encodes provided function parameters into `BuilderData` containing ABI contract call
    pub fn encode_input(&self, tokens: &[Token], pair: Option<&Keypair>) -> Result<BuilderData, SerializationError>
    {
		let params = self.input_params();

		if !Token::types_check(tokens, params.as_slice()) {
			return Err(SerializationError::WrongParameterType);
		}

		if self.signed && pair.is_none() {
			return Err(SerializationError::KeyPairNeeded);
		}

        // prepare standard message
        let mut builder = BuilderData::new();
		for token in tokens.iter().rev() {
			builder = token.prepend_to(builder);
			//println!("{}", builder);
		}

		if self.signed {
			// if all references are used in root cell then expand cells chain with new root
			// to put signature cell reference there
			if BuilderData::references_capacity() == builder.references_used() {
				let mut new_builder = BuilderData::new();
				new_builder.append_reference(builder);
				builder = new_builder;
			};		
		}

        builder = prepend_data_to_chain(builder, {
            // make prefix with ABI version and function ID
            let mut vec = vec![ABI_VERSION];
            vec.extend_from_slice(&self.get_function_id()[..]);
            let len = vec.len() * 8;
            Bitstring::create(vec, len)
        });

		if self.signed {
			let bag = BagOfCells::with_root(builder.clone().into());
			let hash = bag.get_repr_hash_by_index(0).unwrap();
			let signature = pair.unwrap().sign::<Sha512>(hash.as_slice()).to_bytes().to_vec();
			let len = signature.len() * 8;
			builder.prepend_reference(BuilderData::with_raw(signature, len));	
		}

        Ok(builder)
    }
}
/*
#[cfg(test)]
mod tests {
	use {Token, Param, Function, ParamType};

	#[test]
	fn test_function_encode_call() {
		let interface = Function {
			name: "baz".to_owned(),
			inputs: vec![Param {
				name: "a".to_owned(),
				kind: ParamType::Uint(32),
			}, Param {
				name: "b".to_owned(),
				kind: ParamType::Bool,
			}],
			outputs: vec![],
			constant: false,
		};

		let func = Function::from(interface);
		let mut uint = [0u8; 32];
		uint[31] = 69;
		let encoded = func.encode_input(&[Token::Uint(uint.into()), Token::Bool(true)]).unwrap();
		let expected = hex!("cdcd77c000000000000000000000000000000000000000000000000000000000000000450000000000000000000000000000000000000000000000000000000000000001").to_vec();
		assert_eq!(encoded, expected);
	}
}
*/