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
    Contract, MessageId, NodeClient, OrderBy, SortDirection, TransactionId,
    contract::ShardDescr,
    error::SdkError,
    types::{BLOCKS_TABLE_NAME, MASTERCHAIN_ID, BlockId},
};

use ton_types::{fail, error, Result};
use ton_block::MsgAddressInt;

#[derive(Deserialize, Debug, Clone)]
pub struct MsgDescr {
    pub msg_id: Option<MessageId>,
    pub transaction_id: Option<TransactionId>
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
    pub async fn find_last_shard_block(client: &NodeClient, address: &MsgAddressInt) -> Result<BlockId> {
        let workchain = address.get_workchain_id();

        // if account resides in masterchain then starting point is last masterchain block
        // generated before message was sent
        let blocks = client.query(
            BLOCKS_TABLE_NAME,
            &json!({
                "workchain_id": { "eq": MASTERCHAIN_ID }
            }).to_string(),
            "id master { shard_hashes { workchain_id shard descr { root_hash } } }",
            Some(OrderBy {
                path: "seq_no".to_owned(),
                direction: SortDirection::Descending
            }),
            Some(1),
            None
        ).await?;
        //println!("Last block {}", blocks[0]["id"]);

        if MASTERCHAIN_ID == workchain {
            // if account resides in masterchain then starting point is last masterchain block
            blocks[0]["id"]
                .as_str()
                .map(|val| val.to_owned().into())
                .ok_or(SdkError::NotFound("No masterchain block found".to_owned()).into())
        } else {
            // if account is from other chains then starting point is last account's shard block
            // To obtain it we take masterchain block to get shards configuration and select matching shard
            if blocks[0].is_null() {
                // Node SE case - no masterchain, no sharding. Check that only one shard
                let blocks = client.query(
                    BLOCKS_TABLE_NAME,
                    &json!({
                        "workchain_id": { "eq": workchain },
                    }).to_string(),
                    "after_merge shard",
                    Some(OrderBy {
                        path: "seq_no".to_owned(),
                        direction: SortDirection::Descending
                    }),
                    Some(1),
                    None)
                    .await
                    .map_err(|err|  match err.downcast_ref::<SdkError>() {
                        Some(SdkError::WaitForTimeout) => 
                            SdkError::NotFound(format!(
                                "No blocks for workchain {} found", workchain)).into(),
                        _ => err
                    })?;

                if blocks[0].is_null() {
                    fail!(SdkError::NotFound(format!(
                        "No blocks for workchain {} found", workchain)));
                }
                // if workchain is sharded then it is not Node SE and masterchain blocks missing is error
                if blocks[0]["after_merge"] == true || blocks[0]["shard"] != "8000000000000000" {
                    fail!(SdkError::NotFound("No masterchain block found".to_owned()));
                }

                // Take last block by seq_no
                let blocks = client.query(
                    BLOCKS_TABLE_NAME,
                    &json!({
                        "workchain_id": { "eq": workchain },
                        "shard": { "eq": "8000000000000000" },
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
                    .ok_or(SdkError::NotFound("No starting Node SE block found".to_owned()).into())
            } else {
                let shards = blocks[0]["master"]["shard_hashes"]
                    .as_array()
                    .ok_or(SdkError::InvalidData {
                        msg: "No `shard_hashes` field in masterchain block".to_owned()
                    })?;

                let shard_block = Contract::find_matching_shard(shards, address)?;
                
                shard_block["descr"]["root_hash"]
                    .as_str()
                    .map(|val| val.to_owned().into())
                    .ok_or(SdkError::InvalidData {
                        msg: "No `root_hash` field in shard descr".to_owned() }.into()) 
            }
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
        println!("{}: block recieved {:#}", crate::Contract::now(), block);

        if block["after_split"] == true && !Contract::check_shard_match(block.clone(), address)? {
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