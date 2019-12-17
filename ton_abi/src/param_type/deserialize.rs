/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

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
