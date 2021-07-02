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
 *
 */

use std::sync::Arc;

use serde::de::Error;
use serde_json::Value;
use std::fmt;

use crate::error::ClientResult;
use crate::net::{query_collection, OrderBy, ParamsOfQueryCollection, SortDirection};
use crate::ClientContext;
use serde::Serializer;
use ton_block::ShardIdent;

pub const BLOCK_TRAVERSE_FIELDS: &str = r#"
    id
    gen_utime
    workchain_id
    shard
    after_split
    after_merge
    prev_ref {
        root_hash
    }
    prev_alt_ref {
        root_hash
    }
"#;

const BLOCK_MASTER_FIELDS: &str = r#"
    id
    gen_utime
    workchain_id
    shard
    after_split
    after_merge
    prev_ref {
        root_hash
    }
    prev_alt_ref {
        root_hash
    }
    seq_no
    master {
        shard_hashes {
            workchain_id
            shard
            descr {
                gen_utime
                seq_no
                root_hash
            }
        }
    }
"#;

pub(crate) const BLOCK_TRANSACTIONS_FIELDS: &str = r#"
    account_blocks {
        account_addr
        transactions {
            transaction_id
        }
    }
"#;

pub(crate) struct ShardIdentFields<'a>(&'a Value);

impl<'a> ShardIdentFields<'a> {
    pub fn workchain_id(&self) -> i32 {
        self.0["workchain_id"].as_i64().unwrap_or(0) as i32
    }
    pub fn shard(&self) -> &str {
        self.0["shard"].as_str().unwrap_or("")
    }
    pub fn shard_ident(&self) -> ClientResult<ShardIdent> {
        shard_ident(self.workchain_id(), self.shard())
    }
}

pub(crate) struct RefFields<'a>(&'a Value);

impl<'a> RefFields<'a> {
    pub fn root_hash(&self) -> &str {
        self.0["root_hash"].as_str().unwrap_or("")
    }
}

pub(crate) struct BlockFields<'a>(pub &'a Value);

impl<'a> Clone for BlockFields<'a> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<'a> BlockFields<'a> {
    pub fn clone_value(&self) -> Value {
        self.0.clone()
    }

    pub fn id(&self) -> &str {
        self.0["id"].as_str().unwrap_or("")
    }

    pub fn as_shard_ident(&self) -> ShardIdentFields {
        ShardIdentFields(self.0)
    }

    pub fn gen_utime(&self) -> u32 {
        self.0["gen_utime"].as_u64().unwrap_or(0) as u32
    }

    pub fn after_split(&self) -> bool {
        self.0["after_split"].as_bool().unwrap_or(false)
    }

    pub fn after_merge(&self) -> bool {
        self.0["after_merge"].as_bool().unwrap_or(false)
    }

    pub fn prev_ref(&self) -> Option<RefFields> {
        self.0.get("prev_ref").map(|x| RefFields(x))
    }

    pub fn prev_alt_ref(&self) -> Option<RefFields> {
        self.0.get("prev_alt_ref").map(|x| RefFields(x))
    }

    pub fn master(&self) -> Option<MasterFields> {
        self.0.get("master").map(|x| MasterFields(x))
    }

    pub fn account_blocks(&self) -> Option<Vec<AccountBlockFields>> {
        self.0["account_blocks"]
            .as_array()
            .map(|x| x.iter().map(|x| AccountBlockFields(x)).collect())
    }

    fn has_shards(&self) -> bool {
        if let Some(master) = self.master() {
            if let Some(shard_hashes) = master.shard_hashes() {
                for shard_hash in shard_hashes {
                    if let Some(descr) = shard_hash.descr() {
                        if descr.gen_utime() > 0 && descr.seq_no() > 0 {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub(crate) fn get_shards(&self) -> ClientResult<Vec<(ShardIdent, String)>> {
        let mut shards = Vec::new();
        if let Some(master) = self.master() {
            if let Some(shard_hashes) = master.shard_hashes() {
                for shard_hash in shard_hashes {
                    if let Some(descr) = shard_hash.descr() {
                        if descr.gen_utime() > 0 && descr.seq_no() > 0 {
                            shards.push((
                                shard_hash.as_shard_ident().shard_ident()?,
                                descr.root_hash().to_string(),
                            ))
                        }
                    }
                }
            }
        }
        Ok(shards)
    }
}

pub(crate) struct AccountBlockTransactionFields<'a>(&'a Value);

impl<'a> AccountBlockTransactionFields<'a> {
    pub fn transaction_id(&self) -> &str {
        self.0["transaction_id"].as_str().unwrap_or("")
    }
}

pub(crate) struct AccountBlockFields<'a>(&'a Value);

impl<'a> AccountBlockFields<'a> {
    pub fn account_addr(&self) -> &str {
        self.0["account_addr"].as_str().unwrap_or("")
    }

    pub fn transactions(&self) -> Option<Vec<AccountBlockTransactionFields>> {
        self.0["transactions"]
            .as_array()
            .map(|x| x.iter().map(|x| AccountBlockTransactionFields(x)).collect())
    }
}

pub(crate) struct DescrFields<'a>(&'a Value);

impl<'a> DescrFields<'a> {
    pub fn gen_utime(&self) -> u32 {
        self.0["gen_utime"].as_u64().unwrap_or(0) as u32
    }

    pub fn seq_no(&self) -> u64 {
        self.0["seq_no"].as_u64().unwrap_or(0)
    }

    pub fn root_hash(&self) -> &str {
        self.0["root_hash"].as_str().unwrap_or("")
    }
}

pub(crate) struct ShardHashFields<'a>(&'a Value);

impl<'a> ShardHashFields<'a> {
    pub fn as_shard_ident(&self) -> ShardIdentFields {
        ShardIdentFields(self.0)
    }
    pub fn descr(&self) -> Option<DescrFields> {
        self.0.get("descr").map(|x| DescrFields(x))
    }
}

pub(crate) struct MasterFields<'a>(&'a Value);

impl<'a> MasterFields<'a> {
    pub fn shard_hashes(&self) -> Option<Vec<ShardHashFields>> {
        self.0["shard_hashes"]
            .as_array()
            .map(|x| x.iter().map(|x| ShardHashFields(x)).collect())
    }
}

pub(crate) struct MasterBlock {}

impl MasterBlock {
    pub async fn query(
        context: &Arc<ClientContext>,
        start_time: Option<u32>,
        fields: &str,
    ) -> ClientResult<Value> {
        let mut blocks = if let Some(time) = start_time {
            Self::query_blocks(
                context,
                json!({
                    "workchain_id": { "eq": -1 },
                    "gen_utime": { "le": time },
                }),
                SortDirection::DESC,
                1,
                fields,
            )
            .await?
        } else {
            vec![Value::Null]
        };

        let mut last_gen_utime = 0;
        while !blocks.is_empty() {
            while let Some(block) = blocks.pop() {
                let fields = BlockFields(&block);
                if fields.has_shards() {
                    return Ok(block);
                }
                last_gen_utime = fields.gen_utime();
            }
            blocks = Self::query_blocks(
                context,
                json!({
                    "workchain_id": { "eq": -1 },
                    "gen_utime": { "gt": last_gen_utime },
                }),
                SortDirection::ASC,
                10,
                fields,
            )
            .await?;
            blocks.reverse();
        }

        Err(crate::net::Error::invalid_server_response(
            "missing master blocks",
        ))
    }

    async fn query_blocks(
        context: &Arc<ClientContext>,
        filter: Value,
        direction: SortDirection,
        limit: u32,
        fields: &str,
    ) -> ClientResult<Vec<Value>> {
        query_collection(
            context.clone(),
            ParamsOfQueryCollection {
                collection: "blocks".to_string(),
                filter: Some(filter),
                order: Some(vec![OrderBy {
                    path: "gen_utime".to_string(),
                    direction,
                }]),
                result: format!("{} {}", BLOCK_MASTER_FIELDS, fields),
                limit: Some(limit),
            },
        )
        .await
        .map(|x| x.result)
    }
}

fn shard_ident(workchain_id: i32, hex_prefix: &str) -> ClientResult<ShardIdent> {
    let prefix =
        u64::from_str_radix(hex_prefix, 16).map_err(|e| crate::client::Error::internal_error(e))?;
    Ok(ShardIdent::with_tagged_prefix(workchain_id, prefix)
        .map_err(|e| crate::client::Error::internal_error(e))?)
}

pub(crate) fn shard_ident_parse(s: &str) -> ClientResult<ShardIdent> {
    let (workchain_id, tail) = match s.find(":") {
        Some(colon_pos) => {
            let workchain_id = i32::from_str_radix(&s[..colon_pos], 10)
                .map_err(|e| crate::client::Error::internal_error(e))?;
            (workchain_id, &s[colon_pos + 1..])
        }
        None => (0, s),
    };
    shard_ident(workchain_id, tail)
}

pub(crate) fn shard_ident_to_string(shard: &ShardIdent) -> String {
    format!(
        "{}:{:016x}",
        shard.workchain_id(),
        shard.shard_prefix_with_tag()
    )
}

pub(crate) fn serialize_shard_ident<S>(
    shard_ident: &ShardIdent,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&shard_ident_to_string(&shard_ident))
}

struct StringVisitor;

impl<'de> serde::de::Visitor<'de> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("String")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok("null".to_owned())
    }

    fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        d.deserialize_string(StringVisitor)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok("null".to_owned())
    }
}

pub(crate) fn deserialize_shard_ident<'de, D>(d: D) -> Result<ShardIdent, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string = d.deserialize_str(StringVisitor)?;
    shard_ident_parse(&string)
        .map_err(|err| D::Error::custom(format!("Error parsing shard ident: {}", err)))
}
