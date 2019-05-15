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
	use Param;

	#[test]
	fn test_param_type_signature() {
		assert_eq!(ParamType::Uint(256).type_signature(), "uint256".to_owned());
		assert_eq!(ParamType::Int(64).type_signature(), "int64".to_owned());
		assert_eq!(ParamType::Dint.type_signature(), "dint".to_owned());
		assert_eq!(ParamType::Duint.type_signature(), "duint".to_owned());
		assert_eq!(ParamType::Bool.type_signature(), "bool".to_owned());
		assert_eq!(ParamType::Array(Box::new(ParamType::Bool)).type_signature(), "bool[]".to_owned());
		assert_eq!(ParamType::FixedArray(Box::new(ParamType::Int(33)), 2).type_signature(), "int33[2]".to_owned());
		assert_eq!(ParamType::FixedArray(Box::new(ParamType::Array(Box::new(ParamType::Bool))), 2).type_signature(), "bool[][2]".to_owned());
		assert_eq!(ParamType::Bits(256).type_signature(), "bits256".to_owned());
		assert_eq!(ParamType::Bitstring.type_signature(), "bitstring".to_owned());

		let mut tuple_params = vec![];
		tuple_params.push(Param {name: "a".to_owned(), kind: ParamType::Uint(123)});
		tuple_params.push(Param {name: "b".to_owned(), kind: ParamType::Dint});

		assert_eq!(ParamType::Tuple(tuple_params.clone()).type_signature(), "(uint123,dint)".to_owned());
		assert_eq!(ParamType::Array(Box::new(ParamType::Tuple(tuple_params.clone()))).type_signature(), "(uint123,dint)[]".to_owned());
		assert_eq!(ParamType::FixedArray(Box::new(ParamType::Tuple(tuple_params)), 4).type_signature(), "(uint123,dint)[4]".to_owned());
	}
}
