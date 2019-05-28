use {ParamType};

/// Used to convert param type represented as a string to rust structure.
pub struct Reader;

#[derive(Debug)]
pub enum ReaderError{
	InvalidName(String),
}

impl Reader {
	/// Converts string to param type.
	pub fn read(name: &str) -> Result<ParamType, ReaderError> {
		// check if it is a fixed or dynamic array.
		if let Some(']') = name.chars().last() {
			// take number part
			let num: String = name.chars()
				.rev()
				.skip(1)
				.take_while(|c| *c != '[')
				.collect::<String>()
				.chars()
				.rev()
				.collect();

			let count = name.chars().count();
			if num.is_empty() {
				// we already know it's a dynamic array!
				let subtype = Reader::read(&name[..count - 2])?;
				return Ok(ParamType::Array(Box::new(subtype)));
			} else {
				// it's a fixed array.
				let len = usize::from_str_radix(&num, 10)
					.map_err(|_| ReaderError::InvalidName(name.to_owned()))?;
					
				let subtype = Reader::read(&name[..count - num.len() - 2])?;
				return Ok(ParamType::FixedArray(Box::new(subtype), len));
			}
		}

		let result = match name {
			"dint" => ParamType::Dint,
			"duint" => ParamType::Duint,
			"bool" => ParamType::Bool,
			"bitstring" => ParamType::Bitstring,
			// a little trick - here we only recognize parameter as a tuple and fill it 
			// with parameters in `Param` type deserialization
			"tuple" => ParamType::Tuple(Vec::new()),
			s if s.starts_with("int") => {
				let len = usize::from_str_radix(&s[3..], 10)
					.map_err(|_| ReaderError::InvalidName(name.to_owned()))?;
				ParamType::Int(len)
			},
			s if s.starts_with("uint") => {
				let len = usize::from_str_radix(&s[4..], 10)
					.map_err(|_| ReaderError::InvalidName(name.to_owned()))?;
				ParamType::Uint(len)
			},
			s if s.starts_with("bits") => {
				let len = usize::from_str_radix(&s[4..], 10)
					.map_err(|_| ReaderError::InvalidName(name.to_owned()))?;
				ParamType::Bits(len)
			},
			_ => {
				return Err(ReaderError::InvalidName(name.to_owned()));
			}
		};

		Ok(result)
	}
}
