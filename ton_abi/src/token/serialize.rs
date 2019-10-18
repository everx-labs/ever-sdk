use super::*;
use crate::error::*;

use tvm::stack::{BuilderData, IBitstring};
use tvm::block::Serializable;
use tvm::stack::dictionary::HashmapE;

use num_bigint::{BigInt, Sign};

impl TokenValue {
    pub fn pack_values_into_chain(tokens: &[Token], mut cells: Vec<BuilderData>) -> AbiResult<BuilderData> {
        for token in tokens {
            cells.append(&mut token.value.write_to_cells()?);
        }
        Self::pack_cells_into_chain(cells)
    }

    pub fn pack_into_chain(&self) -> AbiResult<BuilderData> {
        Self::pack_cells_into_chain(self.write_to_cells()?)
    }

    // first cell is resulting builder
    // every next cell: put data to root
    fn pack_cells_into_chain(mut cells: Vec<BuilderData>) -> AbiResult<BuilderData> {
        cells.reverse();
        let mut packed_cells = match cells.pop() {
            Some(cell) => vec![cell],
            None => bail!(AbiErrorKind::InvalidData("No cells".to_owned()))
        };
        while let Some(cell) = cells.pop() {
            let builder = packed_cells.last_mut().unwrap();
            if builder.bits_free() < cell.bits_used() || builder.references_free() <= cell.references_used() {
                packed_cells.push(cell);
            } else {
                builder.append_builder(&cell)?;
            }
        }
        while let Some(cell) = packed_cells.pop() {
            match packed_cells.last_mut() {
                Some(builder) => builder.append_reference(cell),
                None => return Ok(cell)
            }
        }
        bail!(AbiErrorKind::NotImplemented)
    }


    fn write_to_cells(&self) -> AbiResult<Vec<BuilderData>> {
        match self {
            TokenValue::Uint(uint) => Self::write_uint(uint),
            TokenValue::Int(int) => Self::write_int(int),
            TokenValue::Bool(b) => Self::write_bool(b),
            TokenValue::Tuple(ref tokens) => {
                let mut vec = vec![];
                for token in tokens.iter() {
                    vec.append(&mut token.value.write_to_cells()?);
                }
                Ok(vec)
            }
            TokenValue::Array(ref tokens) => Self::write_array(tokens),
            TokenValue::FixedArray(ref tokens) => Self::write_fixed_array(tokens),
            TokenValue::Cell(cell) => Self::write_cell(cell),
            TokenValue::Map(key_type, value) => Self::write_map(key_type, value),
            TokenValue::Address(address) => Ok(vec![address.write_to_new_cell()?]),
            TokenValue::Bytes(ref arr) | TokenValue::FixedBytes(ref arr) => Self::write_bytes(arr.to_vec()),
            TokenValue::Gram(gram) => Ok(vec![gram.write_to_new_cell()?]),
        }
    }

    fn write_int(value: &Int) -> AbiResult<Vec<BuilderData>> {
        let vec = value.number.to_signed_bytes_be();
        let vec_bits_length = vec.len() * 8;

        let mut builder = BuilderData::new();

        if value.size > vec_bits_length {
            let padding = if value.number.sign() == num_bigint::Sign::Minus {
                0xFFu8
            } else {
                0u8
            };

            let dif = value.size - vec_bits_length;

            let mut vec_padding = Vec::new();
            vec_padding.resize(dif / 8 + 1, padding);

            builder.append_raw(&vec_padding, dif)?;
            builder.append_raw(&vec, value.size - dif)?;
        } else {
            let offset = vec_bits_length - value.size;
            let first_byte = vec[offset / 8] << offset % 8;

            builder.append_raw(&[first_byte], 8 - offset % 8)?;
            builder.append_raw(&vec[offset / 8 + 1..], vec[offset / 8 + 1..].len() * 8)?;
        };

        Ok(vec![builder])
    }

    fn write_uint(value: &Uint) -> AbiResult<Vec<BuilderData>> {
        let int = Int{
            number: BigInt::from_biguint(Sign::Plus, value.number.clone()),
            size: value.size};

        Self::write_int(&int)
    }

    fn write_bool(value: &bool) -> AbiResult<Vec<BuilderData>> {
        let mut builder = BuilderData::new();
        builder.append_bit_bool(value.clone())?;
        Ok(vec![builder])
    }

    fn write_cell(cell: &Arc<CellData>) -> AbiResult<Vec<BuilderData>> {
        let mut builder = BuilderData::new();
        builder.append_reference_cell(cell.clone());
        Ok(vec![builder])
    }

    // creates dictionary with indexes of an array items as keys and items as values
    // and prepends dictionary to cell
    fn put_array_into_dictionary(array: &[TokenValue]) -> AbiResult<HashmapE> {
        let mut map = HashmapE::with_bit_len(32);

        for i in 0..array.len() {
            let index = (i as u32).write_to_new_cell()?;

            let data = Self::pack_cells_into_chain(array[i].write_to_cells()?)?;

            map.set(index.into(), &data.into())?;
        }

        Ok(map)
    }

    fn write_array(value: &Vec<TokenValue>) -> AbiResult<Vec<BuilderData>> {
        let map = Self::put_array_into_dictionary(value)?;

        let mut builder = BuilderData::new();
        builder.append_u32(value.len() as u32)?;
        
        map.write_to(&mut builder)?;

        Ok(vec![builder])
    }

    fn write_fixed_array(value: &Vec<TokenValue>) -> AbiResult<Vec<BuilderData>> {
        let map = Self::put_array_into_dictionary(value)?;

        Ok(vec![map.write_to_new_cell()?])
    }

    fn write_bytes(mut data: Vec<u8>) -> AbiResult<Vec<BuilderData>> {
        let cell_len = BuilderData::bits_capacity() / 8;
        let mut len = data.len();
        let mut builder = BuilderData::new();
        while len > cell_len {
            len -= cell_len;
            builder.append_raw(&data.split_off(len), cell_len * 8)?;
            let cell = builder.into();
            builder = BuilderData::new();
            builder.append_reference_cell(cell);
        }
        builder.append_raw(&data, len * 8)?;
        let cell = builder.into();
        builder = BuilderData::new();
        builder.append_reference_cell(cell);
        Ok(vec![builder])
    }

    fn write_map(key_type: &ParamType, value: &HashMap<String, TokenValue>) -> AbiResult<Vec<BuilderData>> {
        let bit_len = match key_type {
            ParamType::Int(size) | ParamType::Uint(size) => *size,
            _ => bail!(AbiErrorKind::InvalidData("Only int and uint types can be map keys".to_owned()))
        };
        let mut hashmap = HashmapE::with_bit_len(bit_len);

        for (key, value) in value.iter() {
            let key = Tokenizer::tokenize_parameter(key_type, &serde_json::from_str(key)?)?;

            let mut key_vec = key.write_to_cells()?;
            if key_vec.len() != 1 {
                bail!(AbiErrorKind::InvalidData("Map key must 1-cell length".to_owned()))
            };

            let data = Self::pack_cells_into_chain(value.write_to_cells()?)?;

            hashmap.set(key_vec.pop().unwrap().into(), &data.into())?;
        }

        let mut builder = BuilderData::new();
        hashmap.write_to(&mut builder)?;

        Ok(vec![builder])
    }
}

#[test]
fn test_pack_cells() {
    let cells = vec![
        BuilderData::with_bitstring(vec![1, 2, 0x80]).unwrap(),
        BuilderData::with_bitstring(vec![3, 4, 0x80]).unwrap(),
    ];
    let builder = BuilderData::with_bitstring(vec![1, 2, 3, 4, 0x80]).unwrap();
    assert_eq!(TokenValue::pack_cells_into_chain(cells).unwrap(), builder);

    let cells = vec![
        BuilderData::with_raw(vec![0x55; 100], 100 * 8).unwrap(),
        BuilderData::with_raw(vec![0x55; 127], 127 * 8).unwrap(),
        BuilderData::with_raw(vec![0x55; 127], 127 * 8).unwrap(),
    ];
    
    let builder = BuilderData::with_raw(vec![0x55; 127], 127 * 8).unwrap();
    let builder = BuilderData::with_raw_and_refs(vec![0x55; 127], 127 * 8, vec![builder.into()]).unwrap();
    let builder = BuilderData::with_raw_and_refs(vec![0x55; 100], 100 * 8, vec![builder.into()]).unwrap();
    let tree = TokenValue::pack_cells_into_chain(cells).unwrap();
    assert_eq!(tree, builder);
}