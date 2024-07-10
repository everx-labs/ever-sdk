/*
* Copyright 2018-2021 EverX Labs Ltd.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific EVERX DEV software governing permissions and
* limitations under the License.
*/

pub(crate) mod block;
pub(crate) mod block_iterator;
pub(crate) mod transaction;
pub(crate) mod transaction_iterator;

#[cfg(test)]
mod tests;

use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::net::{query_collection, ParamsOfQueryCollection};
use rand::RngCore;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait::async_trait]
pub trait ChainIterator {
    async fn next(
        &mut self,
        context: &Arc<ClientContext>,
        limit: u32,
        return_resume_state: bool,
    ) -> ClientResult<ResultOfIteratorNext>;
    fn after_remove(&mut self, context: &Arc<ClientContext>);
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct RegisteredIterator {
    /// Iterator handle.
    ///
    /// Must be removed using `remove_iterator`
    /// when it is no more needed for the application.
    pub handle: u32,
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfIteratorNext {
    /// Iterator handle
    pub iterator: u32,

    /// Maximum count of the returned items.
    ///
    /// If value is missing or is less than 1 the library uses 1.
    pub limit: Option<u32>,

    /// Indicates that function must return the iterator state
    /// that can be used for resuming iteration.
    pub return_resume_state: Option<bool>,
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ResultOfIteratorNext {
    /// Next available items.
    ///
    /// Note that `iterator_next` can return an empty items and `has_more` equals to `true`.
    /// In this case the application have to continue iteration.
    /// Such situation can take place when there is no data yet but
    /// the requested `end_time` is not reached.
    pub items: Vec<Value>,

    /// Indicates that there are more available items in iterated range.
    pub has_more: bool,

    /// Optional iterator state that can be used for resuming iteration.
    ///
    /// This field is returned only if the `return_resume_state` parameter
    /// is specified.
    ///
    /// Note that `resume_state` corresponds to the iteration position
    /// after the returned items.
    pub resume_state: Option<Value>,
}

/// Returns next available items.
///
/// In addition to available items this function returns the `has_more` flag
/// indicating that the iterator isn't reach the end of the iterated range yet.
///
/// This function can return the empty list of available items but
/// indicates that there are more items is available.
/// This situation appears when the iterator doesn't reach iterated range
/// but database doesn't contains available items yet.
///
/// If application requests resume state in `return_resume_state` parameter
/// then this function returns `resume_state` that can be used later to
/// resume the iteration from the position after returned items.
///
/// The structure of the items returned depends on the iterator used.
/// See the description to the appropriated iterator creation function.
#[api_function]
pub async fn iterator_next(
    context: Arc<ClientContext>,
    params: ParamsOfIteratorNext,
) -> ClientResult<ResultOfIteratorNext> {
    let iterator = {
        context
            .net
            .iterators
            .lock()
            .await
            .get(&params.iterator)
            .map(|x| x.clone())
            .ok_or(crate::client::Error::invalid_handle(
                params.iterator,
                "iterator",
            ))?
    };
    let mut locked = iterator.lock().await;
    locked
        .next(
            &context,
            params.limit.unwrap_or(1),
            params.return_resume_state.unwrap_or(false),
        )
        .await
}

/// Removes an iterator
///
/// Frees all resources allocated in library to serve iterator.
///
/// Application always should call the `remove_iterator` when iterator
/// is no longer required.
#[api_function]
pub async fn remove_iterator(
    context: Arc<ClientContext>,
    params: RegisteredIterator,
) -> ClientResult<()> {
    let iterator = {
        context
            .net
            .iterators
            .lock()
            .await
            .remove(&params.handle)
            .ok_or(crate::client::Error::invalid_handle(
                params.handle,
                "iterator",
            ))?
    };
    iterator.lock().await.after_remove(&context);
    Ok(())
}

async fn register_iterator(
    context: &Arc<ClientContext>,
    iterator: Box<dyn ChainIterator + Sync + Send>,
) -> ClientResult<RegisteredIterator> {
    let handle = rand::thread_rng().next_u32();
    context
        .net
        .iterators
        .lock()
        .await
        .insert(handle, Arc::new(Mutex::new(iterator)));
    Ok(RegisteredIterator { handle })
}

pub(crate) async fn query_by_ids(
    client: &Arc<ClientContext>,
    collection: &str,
    ids: Vec<String>,
    result_fields: &str,
) -> ClientResult<Vec<Value>> {
    let mut items = Vec::new();
    let mut tail_ids = ids;
    while !tail_ids.is_empty() {
        let head_ids = tail_ids
            .splice(..tail_ids.len().min(40), Vec::default())
            .collect::<Vec<String>>();
        let mut head_by_id = HashMap::new();
        let mut query_queue: HashSet<String> = head_ids.iter().cloned().collect();
        while !query_queue.is_empty() {
            let portion_ids: Vec<String> = query_queue.iter().cloned().collect();
            let portion = query_collection(
                client.clone(),
                ParamsOfQueryCollection {
                    collection: collection.to_string(),
                    filter: Some(json!({ "id": { "in": portion_ids } })),
                    result: result_fields.to_string(),
                    ..Default::default()
                },
            )
            .await?
            .result;
            for item in portion {
                let id = item["id"].as_str().ok_or_else(|| {
                    crate::net::Error::invalid_server_response(format!(
                        "required `{}.id` field is missing",
                        collection
                    ))
                })?;
                query_queue.remove(id);
                head_by_id.insert(id.to_string(), item);
            }
        }
        for id in &head_ids {
            items.push(head_by_id.remove(id).ok_or_else(|| {
                crate::net::Error::invalid_server_response(format!(
                    "missing required {}[{}]",
                    collection, id
                ))
            })?);
        }
    }
    Ok(items)
}
