//! Function param.
use serde::{Serialize, Serializer, Deserialize, Deserializer};

use ParamType;

/// Function param.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Param {
	/// Param name.
	pub name: String,
	/// Param type.
	pub kind: ParamType,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct SerdeParam {
	/// Param name.
	pub name: String,
	/// Param type.
	#[serde(rename="type")]
	pub kind: ParamType,
	/// Tuple components
	#[serde(default)]
	pub components: Vec<Param>
}

impl<'a> Deserialize<'a> for Param {
	fn deserialize<D>(deserializer: D) -> Result<Param, D::Error> where D: Deserializer<'a> {
		// A little trick: tuple parameters is described in JSON as addition field `components` but struct `Param`
		// doesn't have such a field and tuple components is stored inside of `ParamType::Tuple` enum.
		// To use automated deserialization instead of manual parameters recognizing we first deserialize parameter
		// into temp struct `SerdeParam` and then if parameter is a tuple repack tuple components 
		// from `SerdeParam::components` into `ParamType::Tuple`
		let serde_param = SerdeParam::deserialize(deserializer)?;

		let mut result = Self {
			name: serde_param.name,
			kind: serde_param.kind,
		};

		result.kind = match result.kind {
			ParamType::Tuple(_) => ParamType::Tuple(serde_param.components),
			ParamType::Array(array_type) => 
				if let ParamType::Tuple(_) = *array_type {
					ParamType::Array(Box::new(ParamType::Tuple(serde_param.components)))
				} else {
					ParamType::Array(array_type)
				},
			ParamType::FixedArray(array_type, size) => 
				if let ParamType::Tuple(_) = *array_type {
					ParamType::FixedArray(Box::new(ParamType::Tuple(serde_param.components)), size)
				} else {
					ParamType::FixedArray(array_type, size)
				},
			_ => result.kind,
		};

		Ok(result)
	}
}


#[cfg(test)]
mod tests {
	use serde_json;
	use {Param, ParamType};

	#[test]
	fn test_simple_param_deserialization() {
		let s = r#"{
			"name": "a",
			"type": "int9"
		}"#;

		let deserialized: Param = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, Param {
			name: "a".to_owned(),
			kind: ParamType::Int(9),
		});
	}
	
	#[test]
	fn test_tuple_param_deserialization() {
		let s = r#"{
			"name": "a",
			"type": "tuple",
			"components" : [
				{
					"name" : "a",
					"type" : "bitstring"
				},
				{
					"name" : "b",
					"type" : "dint"
				}
			]
		}"#;

		let deserialized: Param = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, Param {
			name: "a".to_owned(),
			kind: ParamType::Tuple(vec![
				Param { name: "a".to_owned(), kind: ParamType::Bitstring },
				Param { name: "b".to_owned(), kind: ParamType::Dint },
			]),
		});
	}

	#[test]
	fn test_tuples_array_deserialization() {
		let s = r#"{
			"name": "a",
			"type": "tuple[]",
			"components" : [
				{
					"name" : "a",
					"type" : "bool"
				},
				{
					"name" : "b",
					"type" : "tuple[5]",
					"components" : [
						{
							"name" : "a",
							"type" : "duint"
						},
						{
							"name" : "b",
							"type" : "bits15"
						}
					]
				}
			]
		}"#;

		let deserialized: Param = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, Param {
			name: "a".to_owned(),
			kind: ParamType::Array(Box::new(ParamType::Tuple(vec![
				Param { 
					name: "a".to_owned(),
					kind: ParamType::Bool
				},
				Param {
					name: "b".to_owned(),
					kind: ParamType::FixedArray(
						Box::new(ParamType::Tuple(vec![
							Param { name: "a".to_owned(), kind: ParamType::Duint },
							Param { name: "b".to_owned(), kind: ParamType::Bits(15) },
						])),
						5
					)
				},
			]))),
		});
	}
}
