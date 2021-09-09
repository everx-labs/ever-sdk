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

pub use batch::{batch_query, ParamsOfBatchQuery, ResultOfBatchQuery};
pub(crate) use endpoint::Endpoint;
pub use errors::{Error, ErrorCode};
pub use iterators::block_iterator::{
    create_block_iterator, resume_block_iterator, ParamsOfCreateBlockIterator,
    ParamsOfResumeBlockIterator,
};
pub use iterators::transaction_iterator::{
    create_transaction_iterator, resume_transaction_iterator, ParamsOfCreateTransactionIterator,
    ParamsOfResumeTransactionIterator,
};
pub use iterators::{
    iterator_next, remove_iterator, ChainIterator, ParamsOfIteratorNext, RegisteredIterator,
    ResultOfIteratorNext,
};
pub use queries::{
    aggregate_collection, query, query_collection, query_counterparties, wait_for_collection,
    ParamsOfQuery, ParamsOfWaitForCollection, ResultOfAggregateCollection, ResultOfQuery,
    ResultOfQueryCollection, ResultOfWaitForCollection,
};
pub(crate) use server_link::{EndpointStat, ServerLink, MAX_TIMEOUT};
pub use subscriptions::{
    subscribe_collection, unsubscribe, ParamsOfSubscribeCollection, ResultOfSubscribeCollection,
    ResultOfSubscription, SubscriptionResponseType,
};
pub use ton_gql::{
    AggregationFn, FieldAggregation, GraphQLQueryEvent, OrderBy, ParamsOfAggregateCollection,
    ParamsOfQueryCollection, ParamsOfQueryCounterparties, ParamsOfQueryOperation, PostRequest,
    SortDirection,
};
pub use transaction_tree::{
    query_transaction_tree, MessageNode, ParamsOfQueryTransactionTree,
    ResultOfQueryTransactionTree, TransactionNode,
};
pub use types::{
    NetworkConfig, ACCOUNTS_COLLECTION, BLOCKS_COLLECTION, MESSAGES_COLLECTION,
    TRANSACTIONS_COLLECTION,
};

use crate::client::ClientContext;
use crate::error::ClientResult;

pub(crate) mod batch;
mod endpoint;
mod errors;
mod gql;
pub(crate) mod iterators;
pub(crate) mod queries;
mod server_link;
pub(crate) mod subscriptions;
mod ton_gql;
pub(crate) mod transaction_tree;
pub(crate) mod types;
mod websocket_link;

#[cfg(test)]
mod tests;

/// Suspends network module to stop any network activity
#[api_function]
pub async fn suspend(context: std::sync::Arc<ClientContext>) -> ClientResult<()> {
    context.get_server_link()?.suspend().await;
    Ok(())
}

/// Resumes network module to enable network activity
#[api_function]
pub async fn resume(context: std::sync::Arc<ClientContext>) -> ClientResult<()> {
    context.get_server_link()?.resume().await;
    Ok(())
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfFindLastShardBlock {
    /// Account address
    pub address: String,
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ResultOfFindLastShardBlock {
    /// Account shard last block ID
    pub block_id: String,
}

/// Returns ID of the last block in a specified account shard
#[api_function]
pub async fn find_last_shard_block(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfFindLastShardBlock,
) -> ClientResult<ResultOfFindLastShardBlock> {
    let address = crate::encoding::account_decode(&params.address)?;

    let block_id =
        crate::processing::blocks_walking::find_last_shard_block(&context, &address, None).await?;

    Ok(ResultOfFindLastShardBlock {
        block_id: block_id.to_string(),
    })
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct EndpointsSet {
    /// List of endpoints provided by server
    pub endpoints: Vec<String>,
}

/// Requests the list of alternative endpoints from server
#[api_function]
pub async fn fetch_endpoints(context: std::sync::Arc<ClientContext>) -> ClientResult<EndpointsSet> {
    let client = context.get_server_link()?;

    Ok(EndpointsSet {
        endpoints: client.fetch_endpoint_addresses().await?,
    })
}

/// Sets the list of endpoints to use on reinit
#[api_function]
pub async fn set_endpoints(
    context: std::sync::Arc<ClientContext>,
    params: EndpointsSet,
) -> ClientResult<()> {
    if params.endpoints.len() == 0 {
        return Err(Error::no_endpoints_provided());
    }

    context
        .get_server_link()?
        .set_endpoints(params.endpoints)
        .await;

    Ok(())
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ResultOfGetEndpoints {
    /// Current query endpoint
    pub query: String,
    /// List of all endpoints used by client
    pub endpoints: Vec<String>,
}

/// Requests the list of alternative endpoints from server
#[api_function]
pub async fn get_endpoints(
    context: std::sync::Arc<ClientContext>,
) -> ClientResult<ResultOfGetEndpoints> {
    let server_link = context.get_server_link()?;
    Ok(ResultOfGetEndpoints {
        query: server_link.get_query_endpoint().await?.query_url.clone(),
        endpoints: server_link.get_all_endpoint_addresses().await?,
    })
}
