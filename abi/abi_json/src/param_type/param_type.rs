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
    /// dint: dynamic sized signed integer value.
    Dint,
    /// duint: dynamic sized unsigned integer value.
    Duint,
    /// bool: boolean value.
    Bool,
    /// Tuple: several values combined into tuple.
    Tuple(Vec<Param>),
    /// T[]: dynamic array of elements of the type T.
    Array(Box<ParamType>),
    /// T[k]: dynamic array of elements of the type T.
    FixedArray(Box<ParamType>, usize),
    /// bits<M>: static sized bits sequence.
    Bits(usize),
    /// bitstring: dynamic sized bits sequence.
    Bitstring,
    /// hashmap - values dictionary
    Map(Box<ParamType>, Box<ParamType>),
    /// TON message address
    Address,
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
            ParamType::Dint => "dint".to_owned(),
            ParamType::Duint => "duint".to_owned(),
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
            ParamType::Bits(size) => format!("bits{}", size),
            ParamType::Bitstring => "bitstring".to_owned(),
            ParamType::Map(key_type, value_type) => 
                format!("map({},{})", key_type.type_signature(), value_type.type_signature()),
            ParamType::Address => format!("address"),
        }
    }
}
