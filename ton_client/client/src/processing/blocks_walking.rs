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

use super::Error;
use crate::client::ClientContext;
use crate::error::ApiResult;
use crate::net::{OrderBy, SortDirection};
use std::sync::Arc;
use ton_block::MsgAddressInt;
use ton_block::MASTERCHAIN_ID;
use ton_sdk::types::BLOCKS_TABLE_NAME;

pub async fn find_last_shard_block(
    context: &Arc<ClientContext>,
    address: &MsgAddressInt,
) -> ApiResult<ton_sdk::BlockId> {
    let workchain = address.get_workchain_id();
    let client = context.get_client()?;

    // if account resides in masterchain then starting point is last masterchain block
    // generated before message was sent
    let blocks = client
        .query(
            BLOCKS_TABLE_NAME,
            &json!({
                "workchain_id": { "eq": MASTERCHAIN_ID }
            }),
            "id master { shard_hashes { workchain_id shard descr { root_hash } } }",
            Some(vec![OrderBy {
                path: "seq_no".to_owned(),
                direction: SortDirection::Descending,
            }]),
            Some(1),
            None,
        )
        .await?;
    debug!("Last block {}", blocks[0]["id"]);

    if MASTERCHAIN_ID == workchain {
        // if account resides in masterchain then starting point is last masterchain block
        blocks[0]["id"]
            .as_str()
            .map(|val| val.to_owned().into())
            .ok_or(Error::block_not_found(
                "No masterchain block found".to_owned(),
            ))
    } else {
        // if account is from other chains then starting point is last account's shard block
        // To obtain it we take masterchain block to get shards configuration and select matching shard
        if blocks[0].is_null() {
            // Node SE case - no masterchain, no sharding. Check that only one shard
            let blocks = client
                .query(
                    BLOCKS_TABLE_NAME,
                    &json!({
                        "workchain_id": { "eq": workchain },
                    }),
                    "after_merge shard",
                    Some(vec![OrderBy {
                        path: "seq_no".to_owned(),
                        direction: SortDirection::Descending,
                    }]),
                    Some(1),
                    None,
                )
                .await?;

            if blocks[0].is_null() {
                return Err(Error::block_not_found(format!(
                    "No blocks for workchain {} found",
                    workchain
                )));
            }
            // if workchain is sharded then it is not Node SE and masterchain blocks missing is error
            if blocks[0]["after_merge"] == true || blocks[0]["shard"] != "8000000000000000" {
                return Err(Error::block_not_found(
                    "No masterchain block found".to_owned(),
                ));
            }

            // Take last block by seq_no
            let blocks = client
                .query(
                    BLOCKS_TABLE_NAME,
                    &json!({
                        "workchain_id": { "eq": workchain },
                        "shard": { "eq": "8000000000000000" },
                    }),
                    "id",
                    Some(vec![OrderBy {
                        path: "seq_no".to_owned(),
                        direction: SortDirection::Descending,
                    }]),
                    Some(1),
                    None,
                )
                .await?;
            blocks[0]["id"]
                .as_str()
                .map(|val| val.to_owned().into())
                .ok_or(Error::block_not_found(
                    "No starting Node SE block found".to_owned(),
                ))
        } else {
            let shards =
                blocks[0]["master"]["shard_hashes"]
                    .as_array()
                    .ok_or(Error::invalid_data(
                        "No `shard_hashes` field in masterchain block",
                    ))?;

            let shard_block =
                ton_sdk::Contract::find_matching_shard(shards, address).map_err(|err| {
                    Error::invalid_data(format!("find matching shard failed {}", err))
                })?;
            if shard_block.is_null() {
                return Err(Error::invalid_data(format!(
                    "No matching shard for account {} in block {}",
                    address, blocks[0]["id"]
                )));
            }

            shard_block["descr"]["root_hash"]
                .as_str()
                .map(|val| val.to_owned().into())
                .ok_or(Error::invalid_data("No `root_hash` field in shard descr"))
        }
    }
}

pub async fn wait_next_block(
    context: &Arc<ClientContext>,
    current: &str,
    address: &MsgAddressInt,
    timeout: Option<u32>,
) -> ApiResult<ton_sdk::Block> {
    let client = context.get_client()?;

    let block = client
        .wait_for(
            BLOCKS_TABLE_NAME,
            &json!({
                "prev_ref": {
                    "root_hash": { "eq": current.to_string() }
                },
                "OR": {
                    "prev_alt_ref": {
                        "root_hash": { "eq": current.to_string() }
                    }
                }
            }),
            ton_sdk::BLOCK_FIELDS,
            timeout,
        )
        .await?;
    debug!(
        "{}: block received {:#}",
        context.env.now_ms() / 1000,
        block
    );

    if block["after_split"] == true && !check_shard_match(block.clone(), address)? {
        client
            .wait_for(
                BLOCKS_TABLE_NAME,
                &json!({
                    "id": { "ne": block["id"]},
                    "prev_ref": {
                        "root_hash": { "eq": current.to_string() }
                    }
                }),
                ton_sdk::BLOCK_FIELDS,
                timeout,
            )
            .await
            .and_then(|val| {
                serde_json::from_value(val)
                    .map_err(|err| Error::invalid_data(format!("Can not parse block: {}", err)))
            })
    } else {
        serde_json::from_value(block)
            .map_err(|err| Error::invalid_data(format!("Can not parse block: {}", err)))
    }
}

fn check_shard_match(shard_descr: serde_json::Value, address: &MsgAddressInt) -> ApiResult<bool> {
    ton_sdk::Contract::check_shard_match(shard_descr, address)
        .map_err(|err| Error::can_not_check_block_shard(err))
}
