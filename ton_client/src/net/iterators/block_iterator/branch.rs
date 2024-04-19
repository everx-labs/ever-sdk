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

use std::sync::Arc;

use ever_block::ShardIdent;

use crate::error::ClientResult;
use crate::net::iterators::block::BlockFields;
use crate::net::iterators::block_iterator::filter::Filter;
use crate::net::iterators::block_iterator::NextLink;
use crate::ClientContext;
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone)]
pub struct Branch {
    #[serde(
        serialize_with = "crate::net::iterators::block::serialize_shard_ident"
    )]
    #[serde(
        deserialize_with = "crate::net::iterators::block::deserialize_shard_ident"
    )]
    pub(crate) shard: ShardIdent,
    pub(crate) block_id: String,
    pub(crate) update_time: u64,
    pub(crate) next_link: NextLink,
}

impl Branch {
    pub(crate) fn branches_from_master_block(
        context: &Arc<ClientContext>,
        master_block: Value,
        filter: &Filter,
    ) -> ClientResult<Vec<Self>> {
        let update_time = context.env.now_ms() / 1000;
        let mut branches = Vec::new();
        let fields = BlockFields(&master_block);
        let shard = fields.as_shard_ident().shard_ident()?;
        if filter.match_shard(&shard) {
            branches.push(Branch {
                shard,
                block_id: fields.id().to_string(),
                update_time,
                next_link: NextLink::ByBoth,
            });
        }
        for (shard, block_id) in fields.get_shards()? {
            if filter.match_shard(&shard) {
                branches.push(Branch {
                    shard,
                    block_id,
                    update_time,
                    next_link: NextLink::ByBoth,
                });
            }
        }
        Ok(branches)
    }
}
