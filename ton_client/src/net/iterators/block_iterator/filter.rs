/*
 * Copyright 2018-2021 TON DEV SOLUTIONS LTD.
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

use ton_block::ShardIdent;

use crate::error::ClientResult;
use crate::net::iterators::block::{shard_ident_parse, BlockFields};
use crate::net::iterators::block_iterator::ParamsOfCreateBlockIterator;
use serde_json::Value;

#[derive(Clone)]
pub(crate) struct Filter {
    pub shards: Vec<ShardIdent>,
    pub start_time: Option<u32>,
    pub end_time: Option<u32>,
    pub result_fields: String,
}

impl Filter {
    pub fn from(params: &ParamsOfCreateBlockIterator) -> ClientResult<Self> {
        let shards = match &params.shard_filter {
            Some(shard_filter) => {
                let mut shards = Vec::new();
                for shard in shard_filter {
                    shards.push(shard_ident_parse(shard)?);
                }
                shards
            }
            None => Default::default(),
        };
        Ok(Self {
            shards,
            start_time: params.start_time,
            end_time: params.end_time,
            result_fields: params.result.clone().unwrap_or(String::default()),
        })
    }

    fn match_start_time(&self, time: u32) -> bool {
        match self.start_time {
            Some(start_time) => time >= start_time,
            None => true,
        }
    }

    fn match_end_time(&self, time: u32) -> bool {
        match self.end_time {
            Some(end_time) => time < end_time,
            None => true,
        }
    }

    pub fn match_shard(&self, shard: &ShardIdent) -> bool {
        if self.shards.is_empty() {
            true
        } else {
            self.shards
                .iter()
                .find(|x| x.is_ancestor_for(shard) || shard.is_ancestor_for(x))
                .is_some()
        }
    }

    pub fn is_required_to_traverse(&self, block: &Value) -> ClientResult<bool> {
        let fields = BlockFields(block);
        let shard = fields.as_shard_ident().shard_ident()?;
        let time = fields.gen_utime();
        Ok(self.match_shard(&shard) && self.match_end_time(time))
    }

    /**
     * @param {Block} block
     * @return {boolean}
     */
    pub fn is_required_to_iterate(&self, block: &Value) -> ClientResult<bool> {
        let fields = BlockFields(block);
        let shard = fields.as_shard_ident().shard_ident()?;
        let time = fields.gen_utime();
        Ok(self.match_shard(&shard) && self.match_start_time(time) && self.match_end_time(time))
    }
}
