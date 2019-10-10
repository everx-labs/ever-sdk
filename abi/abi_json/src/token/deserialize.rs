use types::int::{Int, Uint};
use {Param, ParamType};
use serde_json;
use std::sync::Arc;
use super::*;
use crate::error::*;

use num_bigint::{BigInt, BigUint};
use tvm::stack::{CellData, BuilderData, SliceData, IBitstring};
use tvm::stack::dictionary::{HashmapE, HashmapType};
use tvm::block::BlockResult;
use tvm::block::types::Grams;

impl TokenValue {
    /// Deserializes value from `SliceData` to `TokenValue`
    pub fn read_from(param_type: &ParamType, mut cursor: SliceData) -> AbiResult<(Self, SliceData)> {
        match param_type {
            ParamType::Unknown => bail!(AbiErrorKind::DeserializationError(cursor)),
            ParamType::Uint(size) => Self::read_uint(*size, cursor),
            ParamType::Int(size) => Self::read_int(*size, cursor),
            ParamType::Bool => {
                cursor = find_next_bits(cursor, 1)?;
                Ok((TokenValue::Bool(cursor.get_next_bit().unwrap()), cursor))
            }
            ParamType::Tuple(tuple_params) => Self::read_tuple(tuple_params, cursor),
            ParamType::Array(param_type) => Self::read_array(&param_type, cursor),
            ParamType::FixedArray(param_type, size) => {
                Self::read_fixed_array(&param_type, *size, cursor)
            }
            ParamType::Cell => Self::read_cell(cursor)
                .map(|(cell, cursor)| (TokenValue::Cell(cell), cursor)),
            ParamType::Map(key_type, value_type) => Self::read_hashmap(key_type, value_type, cursor),
            ParamType::Address => {
                let original = cursor.clone();
                cursor = find_next_bits(cursor, 1)?;
                match <MsgAddress as tvm::block::Deserializable>::construct_from(&mut cursor) {
                    Ok(address) => Ok((TokenValue::Address(address), cursor)),
                    Err(_) => bail!(AbiErrorKind::DeserializationError(original))
                }
            }
            ParamType::Bytes => Self::read_bytes(None, cursor),
            ParamType::FixedBytes(size) => Self::read_bytes(Some(*size), cursor),
            ParamType::Gram => {
                let original = cursor.clone();
                cursor = find_next_bits(cursor, 1)?;
                match <Grams as tvm::block::Deserializable>::construct_from(&mut cursor) {
                    Ok(gram) => Ok((TokenValue::Gram(gram), cursor)),
                    Err(_) => bail!(AbiErrorKind::DeserializationError(original))
                }
            }
        }
    }

    fn read_uint(size: usize, cursor: SliceData) -> AbiResult<(Self, SliceData)> {
        let (vec, cursor) = get_next_bits_from_chain(cursor, size)?;
        let number = BigUint::from_bytes_be(&vec) >> (vec.len() * 8 - size);
        Ok((TokenValue::Uint(Uint { number, size }), cursor))
    }

    fn read_int(size: usize, cursor: SliceData) -> AbiResult<(Self, SliceData)> {
        let (vec, cursor) = get_next_bits_from_chain(cursor, size)?;
        let number = BigInt::from_signed_bytes_be(&vec) >> (vec.len() * 8 - size);
        Ok((TokenValue::Int(Int { number, size }), cursor))
    }

    fn read_tuple(tuple_params: &[Param], cursor: SliceData) -> AbiResult<(Self, SliceData)> {
        let mut tokens = Vec::new();
        let mut cursor = cursor;
        for param in tuple_params {
            let (token_value, new_cursor) = TokenValue::read_from(&param.kind, cursor)?;
            tokens.push(Token {
                name: param.name.clone(),
                value: token_value,
            });
            cursor = new_cursor;
        }
        Ok((TokenValue::Tuple(tokens), cursor))
    }

    fn read_array_from_map(param_type: &ParamType, mut cursor: SliceData, size: usize)
    -> AbiResult<(Vec<Self>, SliceData)> {
        let original = cursor.clone();
        cursor = find_next_bits(cursor, 1)?;
        let map = match cursor.get_dictionary() {
            Ok(data) => HashmapE::with_data(32, data),
            Err(_) => bail!(AbiErrorKind::DeserializationError(original))
        };
        let mut result = vec![];
        for i in 0..size {
            let mut index = BuilderData::new();
            index.append_u32(i as u32).unwrap();
            match map.get(index.into()) {
                Ok(Some(item_slice)) => {
                    let (token, item_slice) = Self::read_from(param_type, item_slice)?;
                    if item_slice.remaining_references() != 0 || item_slice.remaining_bits() != 0 {
                        bail!(AbiErrorKind::IncompleteDeserializationError)
                    }
                    result.push(token);
                }
                _ => bail!(AbiErrorKind::DeserializationError(original))
            }
        }

        Ok((result, cursor))
    }

    fn read_array(param_type: &ParamType, mut cursor: SliceData)
    -> AbiResult<(Self, SliceData)> {
        cursor = find_next_bits(cursor, 32)?;
        let size = cursor.get_next_u32().unwrap();
        let (result, cursor) = Self::read_array_from_map(param_type, cursor, size as usize)?;

        Ok((TokenValue::Array(result), cursor))
    }

    fn read_fixed_array(
        param_type: &ParamType,
        size: usize,
        cursor: SliceData,
    ) -> AbiResult<(Self, SliceData)> {
        let (result, cursor) = Self::read_array_from_map(param_type, cursor, size)?;

        Ok((TokenValue::FixedArray(result), cursor))
    }

    fn read_cell(mut cursor: SliceData) -> AbiResult<(Arc<CellData>, SliceData)> {
        let original = cursor.clone();
        let cell = match cursor.remaining_references() {
            0 => bail!(AbiErrorKind::DeserializationError(original)),
            1 if cursor.cell().references_used() == BuilderData::references_capacity() => {
                cursor = SliceData::from(cursor.reference(0).unwrap());
                match cursor.checked_drain_reference() {
                    Ok(cell) => cell,
                    Err(_) => bail!(AbiErrorKind::DeserializationError(original))
                }
            }
            _ => cursor.checked_drain_reference().unwrap()
        };
        Ok((cell.clone(), cursor))
    }

    fn read_hashmap(key_type: &ParamType, value_type: &ParamType, mut cursor: SliceData)
    -> AbiResult<(Self, SliceData)> {
        let original = cursor.clone();
        cursor = find_next_bits(cursor, 1)?;
        let mut new_map = HashMap::new();
        if cursor.get_next_bit().unwrap() {
            let cell = match cursor.checked_drain_reference() {
                Ok(cell) => cell,
                Err(_) => bail!(AbiErrorKind::DeserializationError(original))
            };
            let hashmap = HashmapE::with_hashmap(key_type.bit_len(), Some(cell));
            let result = hashmap.iterate(&mut |key, value| -> BlockResult<bool> {
                let key = Self::read_from(key_type, key).unwrap().0;
                let value = Self::read_from(value_type, value).unwrap().0;
                let key = serde_json::to_string(&key).unwrap();
                new_map.insert(key, value);
                Ok(true)
            });
            if result.is_err() {
                bail!(AbiErrorKind::DeserializationError(original))
            }
        }
        Ok((TokenValue::Map(value_type.clone(), new_map), cursor))
    }

    fn read_bytes(size: Option<usize>, cursor: SliceData)
    -> AbiResult<(Self, SliceData)> {
        let original = cursor.clone();
        let (mut cell, cursor) = Self::read_cell(cursor)?;

        let mut data = vec![];
        loop {
            data.extend_from_slice(cell.data());
            data.pop();
            cell = match cell.reference(0) {
                Ok(cell) => cell.clone(),
                Err(_) => break
            };
        }
        match size {
            Some(size) => if size == data.len() {
                Ok((TokenValue::FixedBytes(data), cursor))
            } else {
                bail!(AbiErrorKind::DeserializationError(original))
            }
            None => Ok((TokenValue::Bytes(data), cursor))
        }
    }
}

fn get_next_bits_from_chain(mut cursor: SliceData, bits: usize)
-> AbiResult<(Vec<u8>, SliceData)> {
    cursor = find_next_bits(cursor, bits)?;
    Ok((cursor.get_next_bits(bits).unwrap(), cursor))
}

fn find_next_bits(mut cursor: SliceData, bits: usize) -> AbiResult<SliceData> {
    debug_assert!(bits != 0);
    let original = cursor.clone();
    if cursor.remaining_bits() == 0 {
        if cursor.reference(1).is_ok() {
            bail!(AbiErrorKind::IncompleteDeserializationError)
        }
        cursor = match cursor.reference(0) {
            Ok(cell) => cell.into(),
            Err(_) => bail!(AbiErrorKind::DeserializationError(original))
        };
    }
    match cursor.remaining_bits() >= bits  {
        true => Ok(cursor),
        false => bail!(AbiErrorKind::DeserializationError(original))
    }
}
