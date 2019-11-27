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

use int::{Int, Uint};
use {Param, ParamType};
use serde_json;
use std::sync::Arc;
use super::*;
use crate::error::*;

use num_bigint::{BigInt, BigUint};
use ton_vm::stack::{CellData, BuilderData, SliceData, IBitstring};
use ton_types::dictionary::{HashmapE, HashmapType};
use ton_block::types::Grams;

impl TokenValue {
    /// Deserializes value from `SliceData` to `TokenValue`
    pub fn read_from(param_type: &ParamType, mut cursor: SliceData) -> AbiResult<(Self, SliceData)> {
        match param_type {
            ParamType::Unknown => bail!(AbiErrorKind::DeserializationError("Unknown ParamType", cursor)),
            ParamType::Uint(size) => Self::read_uint(*size, cursor),
            ParamType::Int(size) => Self::read_int(*size, cursor),
            ParamType::Bool => {
                cursor = find_next_bits(cursor, 1)?;
                Ok((TokenValue::Bool(cursor.get_next_bit()?), cursor))
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
                cursor = find_next_bits(cursor, 1)?;
                let address = <MsgAddress as ton_block::Deserializable>::construct_from(&mut cursor)?;
                Ok((TokenValue::Address(address), cursor))
            }
            ParamType::Bytes => Self::read_bytes(None, cursor),
            ParamType::FixedBytes(size) => Self::read_bytes(Some(*size), cursor),
            ParamType::Gram => {
                cursor = find_next_bits(cursor, 1)?;
                let gram = <Grams as ton_block::Deserializable>::construct_from(&mut cursor)?;
                Ok((TokenValue::Gram(gram), cursor))
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
        let map = HashmapE::with_data(32, cursor.get_dictionary()?);
        let mut result = vec![];
        for i in 0..size {
            let mut index = BuilderData::new();
            index.append_u32(i as u32)?;
            match map.get(index.into()) {
                Ok(Some(item_slice)) => {
                    let (token, item_slice) = Self::read_from(param_type, item_slice)?;
                    if item_slice.remaining_references() != 0 || item_slice.remaining_bits() != 0 {
                        bail!(AbiErrorKind::IncompleteDeserializationError(original))
                    }
                    result.push(token);
                }
                _ => bail!(AbiErrorKind::DeserializationError("", original))
            }
        }

        Ok((result, cursor))
    }

    fn read_array(param_type: &ParamType, mut cursor: SliceData) -> AbiResult<(Self, SliceData)> {
        cursor = find_next_bits(cursor, 32)?;
        let size = cursor.get_next_u32()?;
        let (result, cursor) = Self::read_array_from_map(param_type, cursor, size as usize)?;

        Ok((TokenValue::Array(result), cursor))
    }

    fn read_fixed_array(param_type: &ParamType, size: usize, cursor: SliceData) -> AbiResult<(Self, SliceData)> {
        let (result, cursor) = Self::read_array_from_map(param_type, cursor, size)?;

        Ok((TokenValue::FixedArray(result), cursor))
    }

    fn read_cell(mut cursor: SliceData) -> AbiResult<(Arc<CellData>, SliceData)> {
        let cell = match cursor.remaining_references() {
            1 if cursor.cell().references_used() == BuilderData::references_capacity() => {
                cursor = SliceData::from(cursor.reference(0)?);
                cursor.checked_drain_reference()?
            }
            _ => cursor.checked_drain_reference()?
        };
        Ok((cell.clone(), cursor))
    }

    fn read_hashmap(key_type: &ParamType, value_type: &ParamType, mut cursor: SliceData)
    -> AbiResult<(Self, SliceData)> {
        cursor = find_next_bits(cursor, 1)?;
        let mut new_map = HashMap::new();
        let hashmap = HashmapE::with_data(32, cursor.get_dictionary()?);
        hashmap.iterate(&mut |key, value| -> AbiResult<bool> {
            let key = Self::read_from(key_type, key)?.0;
            let key = serde_json::to_string(&key)?;
            let value = Self::read_from(value_type, value)?.0;
            new_map.insert(key, value);
            Ok(true)
        })?;
        Ok((TokenValue::Map(value_type.clone(), new_map), cursor))
    }

    fn read_bytes(size: Option<usize>, cursor: SliceData) -> AbiResult<(Self, SliceData)> {
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
            Some(size) if size == data.len() => Ok((TokenValue::FixedBytes(data), cursor)),
            Some(_) => bail!(AbiErrorKind::DeserializationError("Size of fixed bytes is not correspond to expected size", original)),
            None => Ok((TokenValue::Bytes(data), cursor))
        }
    }
}

fn get_next_bits_from_chain(mut cursor: SliceData, bits: usize) -> AbiResult<(Vec<u8>, SliceData)> {
    cursor = find_next_bits(cursor, bits)?;
    Ok((cursor.get_next_bits(bits)?, cursor))
}

fn find_next_bits(mut cursor: SliceData, bits: usize) -> AbiResult<SliceData> {
    debug_assert!(bits != 0);
    let original = cursor.clone();
    if cursor.remaining_bits() == 0 {
        if cursor.reference(1).is_ok() {
            bail!(AbiErrorKind::IncompleteDeserializationError(original))
        }
        cursor = cursor.reference(0)?.into();
    }
    match cursor.remaining_bits() >= bits  {
        true => Ok(cursor),
        false => bail!(AbiErrorKind::DeserializationError("Not enought remaining bits in the cell", original))
    }
}
