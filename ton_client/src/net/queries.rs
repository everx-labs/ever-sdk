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

use serde_json::Value;

use crate::client::ClientContext;
use crate::error::{AddNetworkUrl, ClientResult};
use crate::net::ParamsOfQueryCollection;

use super::Error;

//------------------------------------------------------------------------------------------ query

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfQuery {
    /// GraphQL query text.
    pub query: String,
    /// Variables used in query. Must be a map with named values that
    /// can be used in query.
    pub variables: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ResultOfQuery {
    /// Result provided by DAppServer.
    pub result: Value,
}

/// Performs DAppServer GraphQL query.
#[api_function]
pub async fn query(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfQuery,
) -> ClientResult<ResultOfQuery> {
    let server_link = context.get_server_link()?;
    let query = GraphQLQuery {
        query: params.query,
        variables: params.variables,
    };
    let result = server_link
        .query(query, None, None)
        .await
        .map_err(|err| Error::queries_query_failed(err))
        .add_network_url(server_link)
        .await?;

    let result = serde_json::from_value(result)
        .map_err(|err| Error::queries_query_failed(format!("Can not parse result: {}", err)))
        .add_network_url(server_link)
        .await?;

    Ok(ResultOfQuery { result })
}

//------------------------------------------------------------------------------- query_collection

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ResultOfQueryCollection {
    /// Objects that match the provided criteria
    pub result: Vec<serde_json::Value>,
}

/// Queries collection data
///
/// Queries data that satisfies the `filter` conditions,
/// limits the number of returned records and orders them.
/// The projection fields are limited to `result` fields
#[api_function]
pub async fn query_collection(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfQueryCollection,
) -> ClientResult<ResultOfQueryCollection> {
    let client = context.get_server_link()?;
    let result = client
        .query_collection(params, None)
        .await
        .map_err(|err| Error::queries_query_failed(err))
        .add_network_url(client)
        .await?
        .clone();

    let result = serde_json::from_value(result)
        .map_err(|err| Error::queries_query_failed(format!("Can not parse result: {}", err)))
        .add_network_url(client)
        .await?;

    Ok(ResultOfQueryCollection { result })
}

//---------------------------------------------------------------------------- wait_for_collection

#[derive(Serialize, Deserialize, ApiType, Clone, Default)]
pub struct ParamsOfWaitForCollection {
    /// Collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// Collection filter
    pub filter: Option<serde_json::Value>,
    /// Projection (result) string
    pub result: String,
    /// Query timeout
    pub timeout: Option<u32>,
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ResultOfWaitForCollection {
    /// First found object that matches the provided criteria
    pub result: serde_json::Value,
}

/// Returns an object that fulfills the conditions or waits for its appearance
///
/// Triggers only once.
/// If object that satisfies the `filter` conditions
/// already exists - returns it immediately.
/// If not - waits for insert/update of data within the specified `timeout`,
/// and returns it.
/// The projection fields are limited to `result` fields
#[api_function]
pub async fn wait_for_collection(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfWaitForCollection,
) -> ClientResult<ResultOfWaitForCollection> {
    let client = context.get_server_link()?;
    let result = client
        .wait_for_collection(params, None)
        .await
        .map_err(|err| Error::queries_wait_for_failed(err))
        .add_network_url(client)
        .await?;

    Ok(ResultOfWaitForCollection { result })
}

//--------------------------------------------------------------------------- aggregate_collection

use crate::net::ton_gql::GraphQLQuery;
use crate::net::ParamsOfAggregateCollection;

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ResultOfAggregateCollection {
    /// Values for requested fields.
    ///
    /// Returns an array of strings. Each string refers to the corresponding `fields` item.
    /// Numeric value is returned as a decimal string representations.
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
        .aggregate_collection(params, None)
        .await
        .map_err(|err| Error::queries_query_failed(err))
        .add_network_url(client)
        .await?;

    Ok(ResultOfAggregateCollection { values })
}
