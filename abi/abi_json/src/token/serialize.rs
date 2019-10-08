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

    fn pack_cells_into_chain(mut cells: Vec<BuilderData>) -> AbiResult<BuilderData> {
        if cells.len() == 0 { bail!(AbiErrorKind::InvalidData("No cells".to_owned())) };

        let mut packed_cells = vec![cells.remove(0)];

        cells.reverse();

        while let Some(cell) = cells.pop() {
            let len = packed_cells.len();
            let mut builder = &mut packed_cells[len - 1];

            let need_new_cell = if builder.bits_free() < cell.bits_used(){
                true
            } else {
                if builder.references_free() > cell.references_used() {
                    false
                } else if builder.references_free() == cell.references_used() {
                    let (remaining_bits, remaining_refs) = Self::get_required_storage(&cells);

                    remaining_refs == 0 && remaining_bits <= builder.bits_free()
                }
                else {
                    true
                }
            };

            if need_new_cell {
                packed_cells.push(BuilderData::new());
                let len = packed_cells.len();
                builder = &mut packed_cells[len - 1];
            }

            builder.checked_append_references_and_data(&cell.into())?;
        }

        while packed_cells.len() > 1 {
            let len = packed_cells.len();
            let cell = packed_cells.pop().unwrap();
            packed_cells[len - 2]
                .checked_append_reference(&cell.into())?;
        }

        Ok(packed_cells.pop().unwrap())
    }

    fn get_required_storage(cells: &[BuilderData]) -> (usize, usize) {
        cells
            .iter()
            .fold(
                (0, 0),
                |(bits, refs), cell| (bits + cell.bits_used(), refs + cell.references_used()))
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
            TokenValue::Address(address) => {
                Ok(vec![address.write_to_new_cell()?])
            }
            TokenValue::Bytes(ref _arr) | TokenValue::FixedBytes(ref _arr) => {
                unimplemented!()
            }
            TokenValue::Gram(gram) => {
                Ok(vec![gram.write_to_new_cell()?])
            },
            _ => unimplemented!(),
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
            builder.append_raw(&vec[offset + 1..], vec[offset + 1..].len() * 8)?;
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