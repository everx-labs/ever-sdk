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

use crate::client::ClientContext;
use crate::error::ApiResult;
use super::deserialize_tree_of_cells_from_base64;

mod parser;
mod query;


#[derive(Serialize, Deserialize, Clone, TypeInfo)]
pub struct ParamsOfBocQuery {
    /// BOC encoded as base64
    pub boc: String,
    /// query string
    pub query: String,
}

#[derive(Serialize, Deserialize, Clone, TypeInfo)]
pub struct ResultOfBocQuery {
    /// JSON containing result of the query
    pub result: serde_json::Value,
}

pub(crate) fn query(_context: std::sync::Arc<ClientContext>, params: ParamsOfBocQuery) -> ApiResult<ResultOfBocQuery> {
    let query = parser::CellQuery::parse(params.query)?;
    let cell = deserialize_tree_of_cells_from_base64(params.boc.as_str())?;
    Ok(ResultOfBocQuery {
        result: query::query_cell(&query, &cell)?
    })
}