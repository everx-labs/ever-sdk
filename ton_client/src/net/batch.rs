/*
* Copyright 2018-2021 TON Labs LTD.
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

use super::Error;
use crate::net::ton_gql::ParamsOfQueryOperation;

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfBatchQuery {
    /// List of query operations that must be performed per single fetch.
    pub(crate) operations: Vec<ParamsOfQueryOperation>,
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ResultOfBatchQuery {
    /// Result values for batched queries.
    ///
    /// Returns an array of values. Each value corresponds to `queries` item.
    pub results: Vec<Value>,
}

/// Performs multiple queries per single fetch.
#[api_function]
pub async fn batch_query(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfBatchQuery,
) -> ClientResult<ResultOfBatchQuery> {
    let server_link = context.get_server_link()?;
    let results = server_link
        .batch_query(&params.operations, None)
        .await
        .map_err(|err| Error::queries_query_failed(err))
        .add_network_url(server_link)
        .await?;

    Ok(ResultOfBatchQuery { results })
}
