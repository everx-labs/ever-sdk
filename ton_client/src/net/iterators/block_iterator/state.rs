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

use std::collections::HashSet;

use serde_json::Value;

use crate::error::ClientResult;
use crate::net::iterators::block::BlockFields;
use crate::net::iterators::block_iterator::branch::Branch;
use crate::net::iterators::block_iterator::filter::Filter;
use crate::net::iterators::block_iterator::NextLink;

#[derive(Clone)]
pub(crate) struct State {
    pub(crate) branches: Vec<Branch>,
    pub(crate) visited_merge_blocks: HashSet<String>,
    pub(crate) next: Vec<Value>,
}

impl State {
    pub fn has_more(&self) -> bool {
        !self.next.is_empty() || !self.branches.is_empty()
    }
}

pub(crate) struct StateBuilder<'a> {
    now_ms: u64,
    filter: &'a Filter,
    pub(crate) state: State,
}

impl<'a> StateBuilder<'a> {
    pub fn new(filter: &'a Filter, visited_merge_blocks: &HashSet<String>, now_ms: u64) -> Self {
        Self {
            now_ms,
            filter,
            state: State {
                branches: Vec::new(),
                visited_merge_blocks: visited_merge_blocks.clone(),
                next: Vec::new(),
            },
        }
    }

    pub(crate) fn merge_to(&mut self, block: Value) -> ClientResult<()> {
        let id = BlockFields(&block).id().to_string();
        if self.state.visited_merge_blocks.contains(&id) {
            self.state.visited_merge_blocks.remove(&id);
        } else {
            self.state.visited_merge_blocks.insert(id);
            self.new_wanted_branch(block, None)?;
        }
        Ok(())
    }

    pub(crate) fn split_to_both(&mut self, blocks: Vec<Value>) -> ClientResult<()> {
        for next in blocks {
            self.new_wanted_branch(next, None)?;
        }
        Ok(())
    }

    pub(crate) fn split_to_one(&mut self, branch: &Branch, block: Value) -> ClientResult<()> {
        // Detect if we found it by prev_ref
        let traversed_by_prev = BlockFields(&block)
            .prev_ref()
            .map(|x| x.root_hash() == branch.block_id)
            .unwrap_or(false);

        // Continue waiting for second split branch and reduce traverse filter
        self.state.branches.push(Branch {
            shard: branch.shard.clone(),
            block_id: branch.block_id.clone(),
            update_time: branch.update_time,
            next_link: if traversed_by_prev {
                NextLink::ByPrevAlt
            } else {
                NextLink::ByPrev
            },
        });

        self.new_wanted_branch(block, None)?;
        Ok(())
    }

    pub(crate) fn new_wanted_branch(
        &mut self,
        block: Value,
        next_link: Option<NextLink>,
    ) -> ClientResult<()> {
        if self.filter.is_required_to_traverse(&block)? {
            let fields = BlockFields(&block);
            self.state.branches.push(Branch {
                block_id: fields.id().to_string(),
                update_time: self.now_ms,
                shard: fields.as_shard_ident().shard_ident()?,
                next_link: next_link.unwrap_or(NextLink::ByBoth),
            });
        }
        if self.filter.is_required_to_iterate(&block)? {
            self.state.next.push(block);
        }
        Ok(())
    }
}
