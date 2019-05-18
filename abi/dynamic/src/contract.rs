use std::io;
use std::collections::HashMap;
use std::collections::hash_map::Values;
use serde::{Deserialize, Deserializer};
use serde::de::{Unexpected, Error as SerdeError};
use serde_json;
use {Function, ABIError};

/// API building calls to contracts ABI.
#[derive(Clone, Debug, PartialEq)]
pub struct Contract {
	/// Contract functions.
	pub functions: HashMap<String, Function>,
}

impl<'a> Deserialize<'a> for Contract {
	fn deserialize<D>(deserializer: D) -> Result<Contract, D::Error> where D: Deserializer<'a> {
		// A little trick similar to `Param` deserialization: first deserialize JSON into temporary struct `SerdeContract`
		// containing necessary fields and then repack functions into HashMap
		let serde_contract = SerdeContract::deserialize(deserializer)?;

		if serde_contract.abi_version != 0 {
			return Err(
				<D::Error as SerdeError>::invalid_value(
					Unexpected::Unsigned(serde_contract.abi_version as u64), &"ABI version `0`")
			);
		}

		let mut result = Self {
			functions: HashMap::new(),
		};

		for function in serde_contract.functions {
			result.functions.insert(function.name.clone(), function);
		}

		Ok(result)
	}
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct SerdeContract {
	/// ABI version.
	#[serde(rename="ABI version")]
	pub abi_version: u8,
	/// Contract functions.
	pub functions: Vec<Function>,
}

impl Contract {
	/// Loads contract from json.
	pub fn load<T: io::Read>(reader: T) -> Result<Self, ABIError> {
		serde_json::from_reader(reader).map_err(|serde_error| ABIError::SerdeError(serde_error))
	}

	/// Creates function call builder.
	pub fn function(&self, name: &str) -> Result<&Function, ABIError> {
		self.functions.get(name).ok_or_else(|| ABIError::InvalidName(name.to_owned()))
	}

	/// Iterate over all functions of the contract in arbitrary order.
	pub fn functions(&self) -> Functions {
		Functions(self.functions.values())
	}
}

/// Contract functions interator.
pub struct Functions<'a>(Values<'a, String, Function>);

impl<'a> Iterator for Functions<'a> {
	type Item = &'a Function;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

#[cfg(test)]
mod tests {
	use {Contract, Function, Param, ParamType};
	use std::collections::HashMap;

	const TEST_ABI: &str = r#"
	{
		"ABI version": 0,
		"functions": [{
				"name": "input_and_output",
				"inputs": [
					{"name": "a","type": "uint64"},
					{"name": "b","type": "uint8[]"},
					{"name": "c","type": "bitstring"}
				],
				"outputs": [
					{"name": "a","type": "dint"},
					{"name": "b","type": "bits8"}
				]
			}, {
				"name": "no_output",
				"inputs": [{"name": "a", "type": "uint15"}],
				"outputs": []
			}, {
				"name": "no_input",
				"inputs": [],
				"outputs": [{"name": "a", "type": "duint"}]
			}, {
				"name": "constructor",
				"inputs": [],
				"outputs": [],
				"signed": false
			}, {
				"name": "signed",
				"inputs": [{"name": "a", "type": "bool"}],
				"outputs": [],
				"signed": true
			}]
	}"#;

	#[test]
	fn test_abi_parse() {
		let parsed_contract = Contract::load(TEST_ABI.as_bytes()).unwrap();

		let mut functions = HashMap::new();

		functions.insert(
			"input_and_output".to_owned(),
			Function {
					name: "input_and_output".to_owned(),
					inputs: vec![
						Param { name: "a".to_owned(), kind: ParamType::Uint(64) },
						Param { name: "b".to_owned(), kind: ParamType::Array(Box::new(ParamType::Uint(8))) },
						Param { name: "c".to_owned(), kind: ParamType::Bitstring },
					],
					outputs: vec![
						Param { name: "a".to_owned(), kind: ParamType::Dint },
						Param { name: "b".to_owned(), kind: ParamType::Bits(8) },
					],
					signed: false
			});

		functions.insert(
			"no_output".to_owned(),
			Function {
					name: "no_output".to_owned(),
					inputs: vec![
						Param { name: "a".to_owned(), kind: ParamType::Uint(15) },
					],
					outputs: vec![],
					signed: false
			});

		functions.insert(
			"no_input".to_owned(),
			Function {
					name: "no_input".to_owned(),
					inputs: vec![],
					outputs: vec![
						Param { name: "a".to_owned(), kind: ParamType::Duint },
					],
					signed: false
			});

		functions.insert(
			"constructor".to_owned(),
			Function {
					name: "constructor".to_owned(),
					inputs: vec![],
					outputs: vec![],
					signed: false
			});

		functions.insert(
			"signed".to_owned(),
			Function {
					name: "signed".to_owned(),
					inputs: vec![
						Param { name: "a".to_owned(), kind: ParamType::Bool },
					],
					outputs: vec![],
					signed: true
			});

		let expected_contract = Contract { functions };

		assert_eq!(parsed_contract, expected_contract);
	}

	const TEST_ABI_WRONG_VERSION: &str = r#"
	{
		"ABI version": 1,
		"functions": [{
				"name": "constructor",
				"inputs": [],
				"outputs": [],
				"signed": false
			}]
	}"#;

	#[test]
	fn test_abi_wrong_version() {
		assert!(Contract::load(TEST_ABI_WRONG_VERSION.as_bytes()).is_err());
	}
}