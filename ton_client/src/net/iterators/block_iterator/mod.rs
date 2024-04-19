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
 *
 */

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use branch::Branch;
use filter::Filter;
use state::State;

use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::net::iterators::block::{
    shard_ident_parse, shard_ident_to_string, BlockFields, MasterBlock, RefFields,
    BLOCK_TRAVERSE_FIELDS,
};
use crate::net::iterators::block_iterator::state::StateBuilder;
use crate::net::iterators::{query_by_ids, register_iterator, ResultOfIteratorNext};
use crate::net::{query_collection, ChainIterator, ParamsOfQueryCollection, RegisteredIterator};
use ever_block::ShardIdent;

mod branch;
mod filter;
mod state;

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub(crate) enum NextLink {
    ByBoth = 0,
    ByPrev = 1,
    ByPrevAlt = 2,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResumeState {
    shards: Vec<String>,
    start_time: Option<u32>,
    end_time: Option<u32>,
    result_fields: String,
    branches: Vec<Branch>,
    visited_merge_blocks: HashSet<String>,
    next: Vec<String>,
}

impl ResumeState {
    fn get_shards(&self) -> ClientResult<Vec<ShardIdent>> {
        let mut result = Vec::new();
        for shard in &self.shards {
            result.push(shard_ident_parse(&shard)?)
        }
        Ok(result)
    }
}

#[derive(Clone)]
pub(crate) struct BlockIterator {
    pub filter: Filter,
    pub state: State,
}

impl BlockIterator {
    pub async fn new(
        context: &Arc<ClientContext>,
        params: ParamsOfCreateBlockIterator,
    ) -> ClientResult<Self> {
        let filter = Filter::from(&params)?;
        let master_block =
            MasterBlock::query(context, params.start_time, &filter.result_fields).await?;
        let branches = Branch::branches_from_master_block(context, master_block, &filter)?;
        let branches_blocks = Self::query_blocks(
            context,
            branches.iter().map(|x| x.block_id.clone()).collect(),
            &filter.result_fields,
        )
        .await?;
        let mut next = Vec::new();
        for block in branches_blocks {
            if filter.is_required_to_iterate(&block)? {
                next.push(block);
            }
        }
        Ok(Self {
            filter,
            state: State {
                branches,
                visited_merge_blocks: HashSet::new(),
                next,
            },
        })
    }

    pub(crate) fn get_resume_state(&self) -> ResumeState {
        ResumeState {
            shards: self
                .filter
                .shards
                .iter()
                .map(|x| shard_ident_to_string(x))
                .collect(),
            start_time: self.filter.start_time,
            end_time: self.filter.end_time,
            result_fields: self.filter.result_fields.clone(),
            branches: self.state.branches.clone(),
            visited_merge_blocks: self.state.visited_merge_blocks.clone(),
            next: self
                .state
                .next
                .iter()
                .map(|x| BlockFields(x).id().to_string())
                .collect(),
        }
    }

    pub(crate) fn get_resume_state_value(&self) -> ClientResult<Value> {
        serde_json::to_value(self.get_resume_state()).map_err(|e| {
            crate::client::Error::internal_error(format!(
                "Can't serialize iterator resume state: {}",
                e
            ))
        })
    }

    pub(crate) async fn from_resume_state(
        context: &Arc<ClientContext>,
        resume: ResumeState,
    ) -> ClientResult<Self> {
        let shards = resume.get_shards()?;
        let next = Self::query_blocks(context, resume.next, &resume.result_fields).await?;
        Ok(Self {
            filter: Filter {
                shards,
                start_time: resume.start_time,
                end_time: resume.end_time,
                result_fields: resume.result_fields,
            },
            state: State {
                branches: resume.branches,
                visited_merge_blocks: resume.visited_merge_blocks,
                next,
            },
        })
    }

    pub async fn resume(
        context: &Arc<ClientContext>,
        params: ParamsOfResumeBlockIterator,
    ) -> ClientResult<Self> {
        let resume = ResumeState::deserialize(&params.resume_state).map_err(|e| {
            crate::client::Error::internal_error(format!("Invalid iterator resume state: {}", e))
        })?;
        Self::from_resume_state(context, resume).await
    }

    async fn query_blocks(
        context: &Arc<ClientContext>,
        block_ids: Vec<String>,
        fields: &str,
    ) -> ClientResult<Vec<Value>> {
        query_by_ids(
            context,
            "blocks",
            block_ids,
            &format!("{} {}", BLOCK_TRAVERSE_FIELDS, fields),
        )
        .await
    }

    async fn query_next(&mut self, context: &Arc<ClientContext>) -> ClientResult<()> {
        let mut builder = StateBuilder::new(
            &self.filter,
            &self.state.visited_merge_blocks,
            context.env.now_ms(),
        );

        let mut next_blocks = self
            .query_next_blocks(context, &self.state.branches)
            .await?;

        for branch in &self.state.branches {
            if let Some(mut next) = next_blocks.remove(&branch.block_id) {
                if next.len() > 1 {
                    builder.split_to_both(next)?;
                } else {
                    let next = next.remove(0);
                    let fields = BlockFields(&next);
                    if fields.after_merge() {
                        builder.merge_to(next)?;
                    } else if fields.after_split() {
                        builder.split_to_one(branch, next)?;
                    } else {
                        builder.new_wanted_branch(next, None)?;
                    }
                }
            } else {
                builder.state.branches.push(branch.clone());
            }
        }
        self.state = builder.state;
        Ok(())
    }

    async fn query_next_blocks(
        &self,
        context: &Arc<ClientContext>,
        branches: &Vec<Branch>,
    ) -> ClientResult<HashMap<String, Vec<Value>>> {
        let mut branches: Vec<&Branch> = branches.iter().collect();
        let mut next_blocks: HashMap<String, Vec<Value>> = HashMap::new();
        while !branches.is_empty() {
            let prev_ids = branches
                .splice(..branches.len().min(40), Vec::default())
                .collect::<Vec<_>>();

            let prev_ids_by = |l: NextLink| {
                prev_ids
                    .iter()
                    .filter(|x| x.next_link == NextLink::ByBoth || x.next_link == l)
                    .map(|x| x.block_id.clone())
                    .collect::<Vec<String>>()
            };
            let by_prev_ids = prev_ids_by(NextLink::ByPrev);
            let by_prev_alt_ids = prev_ids_by(NextLink::ByPrevAlt);

            let mut blocks = query_collection(
                context.clone(),
                ParamsOfQueryCollection {
                    collection: "blocks".to_string(),
                    filter: Some(json!({
                        "prev_ref": { "root_hash": { "in": by_prev_ids } },
                        "OR": { "prev_alt_ref": { "root_hash": { "in": by_prev_alt_ids } } },
                    })),
                    result: format!("{} {}", BLOCK_TRAVERSE_FIELDS, self.filter.result_fields),
                    ..Default::default()
                },
            )
            .await?
            .result;

            while !blocks.is_empty() {
                let block_value = blocks.remove(0);
                let block = BlockFields(&block_value);
                let mut try_add = |prev_ref: Option<RefFields>| {
                    if let Some(prev_id) = prev_ref.map(|x| x.root_hash().to_string()) {
                        if let Some(existing) = next_blocks.get_mut(&prev_id) {
                            existing.push(block.clone_value());
                        } else {
                            next_blocks.insert(prev_id.to_string(), vec![block.clone_value()]);
                        }
                    }
                };
                try_add(block.prev_ref());
                try_add(block.prev_alt_ref());
            }
        }
        Ok(next_blocks)
    }
}

#[async_trait::async_trait]
impl ChainIterator for BlockIterator {
    async fn next(
        &mut self,
        context: &Arc<ClientContext>,
        limit: u32,
        return_resume_state: bool,
    ) -> ClientResult<ResultOfIteratorNext> {
        let limit = limit.max(1) as usize;

        if self.state.next.is_empty() {
            self.query_next(context).await?;
        }

        let mut items = Vec::new();
        while items.len() < limit && !self.state.next.is_empty() {
            items.push(self.state.next.remove(0));
        }

        let resume_state = if return_resume_state {
            Some(self.get_resume_state_value()?)
        } else {
            None
        };

        Ok(ResultOfIteratorNext {
            has_more: self.state.has_more(),
            items,
            resume_state,
        })
    }

    fn after_remove(&mut self, _context: &Arc<ClientContext>) {}
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfCreateBlockIterator {
    /// Starting time to iterate from.
    ///
    /// If the application specifies this parameter then the iteration
    /// includes blocks with `gen_utime` >= `start_time`.
    /// Otherwise the iteration starts from zero state.
    ///
    /// Must be specified in seconds.
    pub start_time: Option<u32>,

    /// Optional end time to iterate for.
    ///
    /// If the application specifies this parameter then the iteration
    /// includes blocks with `gen_utime` < `end_time`.
    /// Otherwise the iteration never stops.
    ///
    /// Must be specified in seconds.
    pub end_time: Option<u32>,

    /// Shard prefix filter.
    ///
    /// If the application specifies this parameter and it is not the empty array
    /// then the iteration will include items related to accounts that belongs to
    /// the specified shard prefixes.
    /// Shard prefix must be represented as a string "workchain:prefix".
    /// Where `workchain` is a signed integer and the `prefix` if a hexadecimal
    /// representation if the 64-bit unsigned integer with tagged shard prefix.
    /// For example: "0:3800000000000000".
    pub shard_filter: Option<Vec<String>>,

    /// Projection (result) string.
    ///
    /// List of the fields that must be returned for iterated items.
    /// This field is the same as the `result` parameter of
    /// the `query_collection` function.
    /// Note that iterated items can contains additional fields that are
    /// not requested in the `result`.
    pub result: Option<String>,
}

/// Creates block iterator.
///
/// Block iterator uses robust iteration methods that guaranties that every
/// block in the specified range isn't missed or iterated twice.
///
/// Iterated range can be reduced with some filters:
/// - `start_time` – the bottom time range. Only blocks with `gen_utime`
/// more or equal to this value is iterated. If this parameter is omitted then there is
/// no bottom time edge, so all blocks since zero state is iterated.
/// - `end_time` – the upper time range. Only blocks with `gen_utime`
/// less then this value is iterated. If this parameter is omitted then there is
/// no upper time edge, so iterator never finishes.
/// - `shard_filter` – workchains and shard prefixes that reduce the set of interesting
/// blocks. Block conforms to the shard filter if it belongs to the filter workchain
/// and the first bits of block's `shard` fields matches to the shard prefix.
/// Only blocks with suitable shard are iterated.
///
/// Items iterated is a JSON objects with block data. The minimal set of returned
/// fields is:
/// ```text
/// id
/// gen_utime
/// workchain_id
/// shard
/// after_split
/// after_merge
/// prev_ref {
///     root_hash
/// }
/// prev_alt_ref {
///     root_hash
/// }
/// ```
/// Application can request additional fields in the `result` parameter.
///
/// Application should call the `remove_iterator` when iterator is no longer required.
#[api_function]
pub async fn create_block_iterator(
    context: Arc<ClientContext>,
    params: ParamsOfCreateBlockIterator,
) -> ClientResult<RegisteredIterator> {
    register_iterator(
        &context,
        Box::new(BlockIterator::new(&context, params).await?),
    )
    .await
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfResumeBlockIterator {
    /// Iterator state from which to resume.
    ///
    /// Same as value returned from `iterator_next`.
    pub resume_state: Value,
}

/// Resumes block iterator.
///
/// The iterator stays exactly at the same position where the `resume_state` was caught.
///
/// Application should call the `remove_iterator` when iterator is no longer required.
#[api_function]
pub async fn resume_block_iterator(
    context: Arc<ClientContext>,
    params: ParamsOfResumeBlockIterator,
) -> ClientResult<RegisteredIterator> {
    register_iterator(
        &context,
        Box::new(BlockIterator::resume(&context, params).await?),
    )
    .await
}
