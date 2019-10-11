//! Function param.
use serde::{Deserialize, Deserializer};

use ParamType;

/// Function param.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    /// Param name.
    pub name: String,
    /// Param type.
    pub kind: ParamType,
}

impl Param {
    pub fn new(name: &str, kind: ParamType) -> Self {
        Self {
            name: name.to_string(),
            kind
        }
    }
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
        // A little trick: tuple parameters is described in JSON as addition field `components`
        // but struct `Param` doesn't have such a field and tuple components is stored inside of 
        // `ParamType::Tuple` enum. To use automated deserialization instead of manual parameters
        // recognizing we first deserialize parameter into temp struct `SerdeParam` and then
        // if parameter is a tuple repack tuple components from `SerdeParam::components` 
        // into `ParamType::Tuple`
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
            ParamType::Map(key_type, value_type) => 
                if let ParamType::Tuple(_) = *value_type {
                    ParamType::Map(key_type, Box::new(ParamType::Tuple(serde_param.components)))
                } else {
                   ParamType::Map(key_type, value_type)
                },
            _ => result.kind,
        };

        Ok(result)
    }
}


#[cfg(test)]
#[path = "tests/test_param.rs"]
mod tests;