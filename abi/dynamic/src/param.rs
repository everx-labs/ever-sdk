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

		if let ParamType::Tuple(_) = result.kind {
			result.kind = ParamType::Tuple(serde_param.components);
		};

		Ok(result)
	}
}

impl Serialize for Param {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
	{
		serializer.serialize_str(&self.name)
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use {Param, ParamType};

	#[test]
	fn param_deserialization() {
		let s = r#"{
			"name": "foo",
			"type": "address"
		}"#;

		let deserialized: Param = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, Param {
			name: "foo".to_owned(),
			kind: ParamType::Address,
		});
	}
}
