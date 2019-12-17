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

//! Function and event param types.

use std::fmt;
use Param;

/// Function and event param types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamType {
    Unknown,
    /// uint<M>: unsigned integer type of M bits.
    Uint(usize),
    /// int<M>: signed integer type of M bits.
    Int(usize),
    /// bool: boolean value.
    Bool,
    /// Tuple: several values combined into tuple.
    Tuple(Vec<Param>),
    /// T[]: dynamic array of elements of the type T.
    Array(Box<ParamType>),
    /// T[k]: dynamic array of elements of the type T.
    FixedArray(Box<ParamType>, usize),
    /// cell - tree of cells
    Cell,
    /// hashmap - values dictionary
    Map(Box<ParamType>, Box<ParamType>),
    /// TON message address
    Address,
    /// byte array
    Bytes,
    /// fixed size byte array
    FixedBytes(usize),
    /// Nanograms
    Gram
}

impl fmt::Display for ParamType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.type_signature())
    }
}

impl ParamType {
    /// Returns type signature according to ABI specification
    pub fn type_signature(&self) -> String {
        match self {
            ParamType::Unknown => format!("unknown"),
            ParamType::Uint(size) => format!("uint{}", size),
            ParamType::Int(size) => format!("int{}", size),
            ParamType::Bool => "bool".to_owned(),
            ParamType::Tuple(params) => {
                let mut signature = "".to_owned();
                for param in params {
                    signature += ",";
                    signature += &param.kind.type_signature();
                }
                signature.replace_range(..1, "(");
                signature + ")"
            },
            ParamType::Array(ref param_type) => format!("{}[]", param_type.type_signature()),
            ParamType::FixedArray(ref param_type, size) => 
                format!("{}[{}]", param_type.type_signature(), size),
            ParamType::Cell => "cell".to_owned(),
            ParamType::Map(key_type, value_type) => 
                format!("map({},{})", key_type.type_signature(), value_type.type_signature()),
            ParamType::Address => format!("address"),
            ParamType::Bytes => format!("bytes"),
            ParamType::FixedBytes(size) => format!("fixedbytes{}", size),
            ParamType::Gram => format!("gram"),
        }
    }

    /// Returns type bit_len for hashmap key
    pub fn bit_len(&self) -> usize {
        match self {
            ParamType::Uint(size) => *size,
            ParamType::Int(size) => *size,
            _ => 0
        }
    }
}
