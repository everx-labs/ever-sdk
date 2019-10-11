use std::fmt;
use serde::{Deserialize, Deserializer};
use serde::de::{Error as SerdeError, Visitor};
use super::{ParamType, Reader};

impl<'a> Deserialize<'a> for ParamType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'a> {
        deserializer.deserialize_identifier(ParamTypeVisitor)
    }
}

struct ParamTypeVisitor;

impl<'a> Visitor<'a> for ParamTypeVisitor {
    type Value = ParamType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a correct name of abi-encodable parameter type")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
        Reader::read(value).map_err(|e| SerdeError::custom(e.to_string()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
        self.visit_str(value.as_str())
    }
}
