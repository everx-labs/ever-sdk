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
#[path = "tests/test_contract.rs"]
mod tests;