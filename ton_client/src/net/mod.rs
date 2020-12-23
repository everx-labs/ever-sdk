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
use crate::error::ClientResult;

mod errors;
mod gql;
pub(crate) mod queries;
mod websocket_link;
mod server_link;
mod server_info;
pub(crate) mod subscriptions;
mod types;

pub use errors::{Error, ErrorCode};
pub use gql::{OrderBy, SortDirection};
pub use queries::{
    query, query_collection, wait_for_collection,
    ParamsOfQuery, ParamsOfQueryCollection, ParamsOfWaitForCollection, ResultOfQuery,
    ResultOfQueryCollection, ResultOfWaitForCollection,
};
pub use subscriptions::{
    subscribe_collection, unsubscribe,
    ParamsOfSubscribeCollection, ResultOfSubscribeCollection, ResultOfSubscription,
    SubscriptionResponseType
};
pub use types::{
    NetworkConfig,
    MESSAGES_TABLE_NAME, CONTRACTS_TABLE_NAME, BLOCKS_TABLE_NAME, TRANSACTIONS_TABLE_NAME
};

pub(crate) use server_link::{ServerLink, MAX_TIMEOUT};

#[cfg(test)]
mod tests;

/// Suspends network module to stop any network activity
#[api_function]
pub async fn suspend(
    context: std::sync::Arc<ClientContext>,
) -> ClientResult<()> {
    context.get_server_link()?.suspend().await;
    Ok(())
}

/// Resumes network module to enable network activity
#[api_function]
pub async fn resume(
    context: std::sync::Arc<ClientContext>,
) -> ClientResult<()> {
    context.get_server_link()?.resume().await;
    Ok(())
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfFindLastShardBlock {
    /// Account address
    pub address: String,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
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

    let block_id = crate::processing::blocks_walking::find_last_shard_block(&context, &address)
        .await?;

    Ok(ResultOfFindLastShardBlock {
        block_id: block_id.to_string()
    })
}
