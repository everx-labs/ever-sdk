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

use crate::{
    MessageId, NodeClient, OrderBy, SortDirection, TransactionId,
    json_helper,
    error::SdkError,
    types::{BLOCKS_TABLE_NAME, MASTERCHAIN_ID, StringId},
};

use ton_types::{fail, error, Result};
use ton_block::{AccountIdPrefixFull, MsgAddressInt, ShardIdent};

use serde_json::Value;

pub type BlockId = StringId;

#[derive(Deserialize, Debug, Clone)]
pub struct ShardDescr {
    pub workchain_id: i32,
    #[serde(deserialize_with = "json_helper::deserialize_shard")]
    pub shard: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MsgDescr {
    pub msg_id: MessageId,
    pub transaction_id: TransactionId
}

#[derive(Deserialize, Debug, Clone)]
pub struct Block {
    pub id: BlockId,
    pub gen_utime: u32,
    pub after_split: bool,
    #[serde(flatten)]
    pub shard_descr: ShardDescr,
    pub in_msg_descr: Vec<MsgDescr>
}

const BLOCK_FIELDS: &str = r#"
    id
    gen_utime
    after_split
    workchain_id
    shard
    in_msg_descr {
        msg_id
        transaction_id
    }
"#;

impl Block {
    pub fn check_shard_match(shard_descr: Value, address: &MsgAddressInt) -> Result<bool> {
        let descr: ShardDescr = serde_json::from_value(shard_descr)?;
        let ident = ShardIdent::with_tagged_prefix(descr.workchain_id, descr.shard)?;
        Ok(ident.contains_full_prefix(&AccountIdPrefixFull::prefix(address)?))
    }

    pub fn find_matching_shard(shards: &Vec<Value>, address: &MsgAddressInt) -> Result<Value> {
        for shard in shards {
            if Self::check_shard_match(shard.clone(), address)? {
                return Ok(shard.clone());
            }
        };
        fail!(SdkError::NotFound(format!("No matching shard for account {}", address)))
    }

    pub async fn find_starting_block(client: &NodeClient, send_time: u32, address: &MsgAddressInt) -> Result<BlockId> {
        let workchain = address.get_workchain_id();
        if MASTERCHAIN_ID == workchain {
            // if account resides in masterchain then starting point is last masterchain block
            // generated before message was sent
            let blocks = client.query(
                BLOCKS_TABLE_NAME,
                &json!({
                    "workchain_id": { "eq": MASTERCHAIN_ID },
                    "gen_utime": { "le": send_time, "gt": 0 }
                }).to_string(),
                "id",
                Some(OrderBy {
                    path: "seq_no".to_owned(),
                    direction: SortDirection::Descending
                }),
                Some(1),
                None
            ).await?;
            blocks[0]["id"]
                .as_str()
                .map(|val| val.to_owned().into())
                .ok_or(SdkError::NotFound("No starting masterchain block returned".to_owned()).into())
        } else {
            // if account is from other chains then starting point is some account's shard block
            // generated before message was sent. To obtain it we take masterchain block to get
            // shards configuration and select matching shard
            let blocks = client.query(
                BLOCKS_TABLE_NAME,
                &json!({
                    "workchain_id": { "eq": MASTERCHAIN_ID },
                    "master": {
                        "max_shard_gen_utime": { "le": send_time, "gt": 0 }
                    }
                }).to_string(),
                "id
                master { shard_hashes { workchain_id shard descr { root_hash } } }",
                Some(OrderBy {
                    path: "seq_no".to_owned(),
                    direction: SortDirection::Descending
                }),
                Some(1),
                None
            ).await?;
            if blocks[0].is_null() {
                fail!(SdkError::NotFound("No starting masterchain block returned".to_owned()))
            }
            let shards = blocks[0]["master"]["shard_hashes"]
                .as_array()
                .ok_or(SdkError::InvalidData {
                    msg: "No `shard_hashes` field in masterchain block".to_owned()
                })?;

            let shard_block = Self::find_matching_shard(shards, address)?;
            
            shard_block["root_hash"]
                .as_str()
                .map(|val| val.to_owned().into())
                .ok_or(SdkError::InvalidData {
                    msg: "No `root_hash` field in shard descr".to_owned() }.into())
        }
    }

    pub async fn wait_next_block(
        client: &NodeClient, current: &BlockId, address: &MsgAddressInt, timeout: Option<u32>
    ) -> Result<Block> {
        let block = client.wait_for(
            BLOCKS_TABLE_NAME,
            &json!({
                "prev_ref": {
                    "root_hash": { "eq": current.to_string() }
                }
            }).to_string(),
            BLOCK_FIELDS,
            timeout).await?;

        if block["after_split"] == true && !Self::check_shard_match(block.clone(), address)? {
            client.wait_for(
                BLOCKS_TABLE_NAME,
                &json!({
                    "id": { "ne": block["id"]},
                    "prev_ref": {
                        "root_hash": { "eq": current.to_string() }
                    }
                }).to_string(),
                BLOCK_FIELDS,
                timeout)
                .await
                .and_then(|val| serde_json::from_value(val).map_err(|err| err.into()))
        } else {
            serde_json::from_value(block).map_err(|err| err.into())
        }
    }
}