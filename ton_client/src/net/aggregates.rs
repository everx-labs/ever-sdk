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

use super::{Error};
use crate::client::ClientContext;
use crate::error::{AddNetworkUrl, ClientResult};
use crate::net::gql::FieldAggregation;
use serde_json::Value;

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfAggregateCollection {
    /// Collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// Collection filter.
    pub filter: Option<serde_json::Value>,
    /// Projection (result) string
    pub fields: Option<Vec<FieldAggregation>>,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ResultOfAggregateCollection {
    /// Values for requested fields.
    ///
    /// Returns an array of strings. Each string refers to the corresponding `fields` item.
    /// Numeric values is returned as a decimal string representations.
    pub values: Value,
}

/// Aggregates collection data.
///
/// Aggregates values from the specified `fields` for records
/// that satisfies the `filter` conditions,
#[api_function]
pub async fn aggregate_collection(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfAggregateCollection,
) -> ClientResult<ResultOfAggregateCollection> {
    let client = context.get_server_link()?;
    let values = client
        .aggregate_collection(
            &params.collection,
            &params.filter.unwrap_or(json!({})),
            &params.fields.unwrap_or(vec![]),
        )
        .await
        .map_err(|err| Error::queries_query_failed(err))
        .add_network_url(client)
        .await?;

    Ok(ResultOfAggregateCollection { values })
}
