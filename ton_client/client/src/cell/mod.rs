/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::dispatch::DispatchTable;
use crate::client::ClientContext;
use crate::types::{ApiResult, ApiError};
use serde_json::Value;
use crate::cell::parser::CellQuery;
use crate::cell::query::query_cell;
use ton_types::Cell;

pub(crate) mod query;
pub(crate) mod parser;

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfCellQuery {
    pub cellBase64: String,
    pub query: String,
}

fn deserialize_tree_of_cells_from_base64(b64: &str) -> ApiResult<Cell>
{
    let bytes = base64::decode(&b64)
        .map_err(|err| ApiError::cell_invalid_query(format!("error decode base64: {}", err)))?;

    ton_types::cells_serialization::deserialize_tree_of_cells(&mut bytes.as_slice())
        .map_err(|err| ApiError::cell_invalid_query(format!("BOC read error: {}", err)))
}

pub(crate) fn query(_context: &mut ClientContext, params: ParamsOfCellQuery) -> ApiResult<Value> {
    let query = CellQuery::parse(params.query)?;
    let cell = deserialize_tree_of_cells_from_base64(params.cellBase64.as_str())?;
    query_cell(&query, &cell)
}


pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.spawn("cell.query", query);
}
