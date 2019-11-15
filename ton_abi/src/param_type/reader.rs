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

use {ParamType};
use crate::error::*;

/// Used to convert param type represented as a string to rust structure.
pub struct Reader;

impl Reader {
    /// Converts string to param type.
    pub fn read(name: &str) -> AbiResult<ParamType> {
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
                    .map_err(|_| AbiErrorKind::InvalidName(name.to_owned()))?;
                    
                let subtype = Reader::read(&name[..count - num.len() - 2])?;
                return Ok(ParamType::FixedArray(Box::new(subtype), len));
            }
        }

        let result = match name {
            "bool" => ParamType::Bool,
            // a little trick - here we only recognize parameter as a tuple and fill it 
            // with parameters in `Param` type deserialization
            "tuple" => ParamType::Tuple(Vec::new()),
            s if s.starts_with("int") => {
                let len = usize::from_str_radix(&s[3..], 10)
                    .map_err(|_| AbiErrorKind::InvalidName(name.to_owned()))?;
                ParamType::Int(len)
            },
            s if s.starts_with("uint") => {
                let len = usize::from_str_radix(&s[4..], 10)
                    .map_err(|_| AbiErrorKind::InvalidName(name.to_owned()))?;
                ParamType::Uint(len)
            },
            s if s.starts_with("map(") && s.ends_with(")") => {
                let types: Vec<&str> = name[5..name.len() - 1].split(",").collect();
                if types.len() != 2 {
                    bail!(AbiErrorKind::InvalidName(name.to_owned()));
                }

                let key_type = Reader::read(types[0])?;
                let value_type = Reader::read(types[1])?;

                match key_type
                {
                    ParamType::Int(_) | ParamType::Uint(_) =>
                        ParamType::Map(Box::new(key_type), Box::new(value_type)),
                    _ => bail!(AbiErrorKind::InvalidName(
                            "Only int and uint types can be map keys".to_owned())),
                }
            },
            "cell" => {
                ParamType::Cell
            }
            "address" => {
                ParamType::Address
            }
            "gram" => {
                ParamType::Gram
            }
            "bytes" => {
                ParamType::Bytes
            }
            s if s.starts_with("fixedbytes") => {
                let len = usize::from_str_radix(&s[10..], 10)
                    .map_err(|_| AbiErrorKind::InvalidName(name.to_owned()))?;
                ParamType::FixedBytes(len)
            }
            _ => {
                bail!(AbiErrorKind::InvalidName(name.to_owned()));
            }
        };

        Ok(result)
    }
}
