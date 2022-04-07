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
*/

use super::Error;
use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::net::{
    Endpoint, OrderBy, ParamsOfQueryCollection, ParamsOfWaitForCollection, SortDirection,
    BLOCKS_COLLECTION,
};
use std::sync::Arc;
use ton_block::MsgAddressInt;
use ton_block::MASTERCHAIN_ID;

pub const BLOCK_FIELDS: &str = r#"
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

pub(crate) async fn find_last_shard_block(
    context: &Arc<ClientContext>,
    address: &MsgAddressInt,
    endpoint: Option<Endpoint>,
) -> ClientResult<ton_sdk::BlockId> {
    let workchain = address.get_workchain_id();
    let server_link = context.get_server_link()?;

    // if account resides in masterchain, then starting point is last masterchain block
    // generated before message was sent
    let blocks = server_link
        .query_collection(
            ParamsOfQueryCollection {
                collection: BLOCKS_COLLECTION.to_string(),
                filter: Some(json!({
                    "workchain_id": { "eq": MASTERCHAIN_ID }
                })),
                result: "id master { shard_hashes { workchain_id shard descr { root_hash } } }"
                    .to_string(),
                order: Some(vec![OrderBy {
                    path: "seq_no".to_owned(),
                    direction: SortDirection::DESC,
                }]),
                limit: Some(1),
            },
            endpoint.clone(),
        )
        .await?;
    debug!("Last block {}", blocks[0]["id"]);

    if MASTERCHAIN_ID == workchain {
        // if account resides in masterchain, then starting point is last masterchain block
        blocks[0]["id"]
            .as_str()
            .map(|val| val.to_owned().into())
            .ok_or(Error::block_not_found(
                "No masterchain block found".to_owned(),
            ))
    } else {
        // if account is from other chains, then starting point is last account's shard block
        // To obtain it we take masterchain block to get shards configuration and select matching shard
        if blocks[0].is_null() {
            // TON OS SE case - no masterchain, no sharding. Check that only one shard
            let blocks = server_link
                .query_collection(
                    ParamsOfQueryCollection {
                        collection: BLOCKS_COLLECTION.to_string(),
                        filter: Some(json!({
                        "workchain_id": { "eq": workchain },
                        })),
                        result: "after_merge shard".to_string(),
                        order: Some(vec![OrderBy {
                            path: "seq_no".to_owned(),
                            direction: SortDirection::DESC,
                        }]),
                        limit: Some(1),
                    },
                    endpoint.clone(),
                )
                .await?;

            if blocks[0].is_null() {
                return Err(Error::block_not_found(format!(
                    "No blocks for workchain {} found",
                    workchain
                )));
            }
            // if workchain is sharded, then it is not TON OS SE and masterchain blocks missing is error
            if blocks[0]["after_merge"] == true || blocks[0]["shard"] != "8000000000000000" {
                return Err(Error::block_not_found(
                    "No masterchain block found".to_owned(),
                ));
            }

            // Take last block by seq_no
            let blocks = server_link
                .query_collection(
                    ParamsOfQueryCollection {
                        collection: BLOCKS_COLLECTION.to_string(),
                        filter: Some(json!({
                        "workchain_id": { "eq": workchain },
                        "shard": { "eq": "8000000000000000" },
                        })),
                        result: "id".to_string(),
                        order: Some(vec![OrderBy {
                            path: "seq_no".to_owned(),
                            direction: SortDirection::DESC,
                        }]),
                        limit: Some(1),
                    },
                    endpoint,
                )
                .await?;
            blocks[0]["id"]
                .as_str()
                .map(|val| val.to_owned().into())
                .ok_or(Error::block_not_found(
                    "No starting TON OS SE block found".to_owned(),
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
) -> ClientResult<ton_sdk::Block> {
    let client = context.get_server_link()?;

    let block = client
        .wait_for_collection(
            ParamsOfWaitForCollection {
                collection: BLOCKS_COLLECTION.to_string(),
                filter: Some(json!({
                    "prev_ref": {
                        "root_hash": { "eq": current.to_string() }
                    },
                    "OR": {
                        "prev_alt_ref": {
                            "root_hash": { "eq": current.to_string() }
                        }
                    }
                })),
                result: BLOCK_FIELDS.to_string(),
                timeout,
            },
            None,
        )
        .await?;
    debug!(
        "{}: block received {:#}",
        context.env.now_ms() / 1000,
        block
    );

    if block["after_split"] == true && !check_shard_match(block.clone(), address)? {
        client
            .wait_for_collection(
                ParamsOfWaitForCollection {
                    collection: BLOCKS_COLLECTION.to_string(),
                    filter: Some(json!({
                        "id": { "ne": block["id"]},
                        "prev_ref": {
                            "root_hash": { "eq": current.to_string() }
                        }
                    })),
                    result: BLOCK_FIELDS.to_string(),
                    timeout,
                },
                None,
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

fn check_shard_match(
    shard_descr: serde_json::Value,
    address: &MsgAddressInt,
) -> ClientResult<bool> {
    ton_sdk::Contract::check_shard_match(shard_descr, address)
        .map_err(|err| Error::can_not_check_block_shard(err))
}
