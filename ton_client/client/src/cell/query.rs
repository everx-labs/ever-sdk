/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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

use serde_json::Value;
use crate::types::{ApiResult, ApiError};
use crate::cell::parser::{CellQuery, CellFieldReader, CellValueReader};
use ton_types::{Cell, SliceData, HashmapE, HashmapType};
use ton_block::types::Grams;

fn read_value(slice: &mut SliceData, reader: &CellValueReader) -> ApiResult<Value> {
    Ok(match reader {
        CellValueReader::IntWithSize(size) => {
            let n = slice.get_next_int(*size)
                .map_err(|err| ApiError::cell_invalid_query(err))?;
            Value::String(format!("{}", n))
        }
        CellValueReader::UIntWithSize(size) => {
            let n = slice.get_next_int(*size)
                .map_err(|err| ApiError::cell_invalid_query(err))?;
            Value::String(format!("{}", n))
        }
        CellValueReader::Grams => {
            let n = Grams::read_from_cell(slice)
                .map_err(|err| ApiError::cell_invalid_query(err))?;
            Value::String(format!("{}", n))
        }
        CellValueReader::Dict(fields) => {
            let mut dict = HashmapE::with_bit_len(256);
            dict.read_hashmap_data(slice)
                .map_err(|err| ApiError::cell_invalid_query(err))?;
            let mut count = 0;
            let result = dict.iterate(&mut |key, value| {
                count += 1;
                Ok(true)
            });
            result.map_err(|err| ApiError::cell_invalid_query(err))?;
            Value::from(count)
        }
    })
}

fn read(slice: &mut SliceData, commands: &Vec<CellFieldReader>) -> ApiResult<Value> {
    let mut values = serde_json::Map::new();
    for (index, command) in commands.iter().enumerate() {
        let name = if command.name.is_empty() { format!("{}", index) } else { command.name.clone() };
        values.insert(name, read_value(slice, &command.value)?);
    }
    Ok(Value::Object(values))
}

pub(crate) fn query_cell(query: &CellQuery, cell: &Cell) -> ApiResult<Value> {
    let mut slice = SliceData::from(cell);
    read(&mut slice, &query.commands)
}
