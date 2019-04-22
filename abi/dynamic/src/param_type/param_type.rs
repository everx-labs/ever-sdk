//! Function and event param types.

use std::fmt;
use Param;

/// Function and event param types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParamType {
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
			ParamType::FixedArray(ref param_type, size) => format!("{}[{}]", param_type.type_signature(), size),
			ParamType::Bits(size) => format!("bits{}", size),
			ParamType::Bitstring => "bitstring".to_owned(),
		}
    }
}

#[cfg(test)]
mod tests {
	use ParamType;

	#[test]
	fn test_param_type_display() {
		assert_eq!(format!("{}", ParamType::Address), "address".to_owned());
		assert_eq!(format!("{}", ParamType::Bytes), "bytes".to_owned());
		assert_eq!(format!("{}", ParamType::FixedBytes(32)), "bytes32".to_owned());
		assert_eq!(format!("{}", ParamType::Uint(256)), "uint256".to_owned());
		assert_eq!(format!("{}", ParamType::Int(64)), "int64".to_owned());
		assert_eq!(format!("{}", ParamType::Bool), "bool".to_owned());
		assert_eq!(format!("{}", ParamType::String), "string".to_owned());
		assert_eq!(format!("{}", ParamType::Array(Box::new(ParamType::Bool))), "bool[]".to_owned());
		assert_eq!(format!("{}", ParamType::FixedArray(Box::new(ParamType::String), 2)), "string[2]".to_owned());
		assert_eq!(format!("{}", ParamType::FixedArray(Box::new(ParamType::Array(Box::new(ParamType::Bool))), 2)), "bool[][2]".to_owned());
	}
}
