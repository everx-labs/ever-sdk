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

use crate::boc::internal::deserialize_object_from_boc;
use crate::boc::Error;
use crate::client::ClientContext;
use crate::error::ClientResult;
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ParamsOfParse {
    /// BOC encoded as base64
    pub boc: String,
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ParamsOfParseShardstate {
    /// BOC encoded as base64
    pub boc: String,
    /// Shardstate identifier
    pub id: String,
    /// Workchain shardstate belongs to
    pub workchain_id: i32,
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ResultOfParse {
    /// JSON containing parsed BOC
    pub parsed: Value,
}

/// Parses message boc into a JSON
///
/// JSON structure is compatible with GraphQL API message object
#[api_function]
pub async fn parse_message(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfParse,
) -> ClientResult<ResultOfParse> {
    let object = deserialize_object_from_boc::<ton_block::Message>(&context, &params.boc, "message").await?;

    let set = ton_block_json::MessageSerializationSet {
        block_id: None,
        boc: object.boc.bytes("message")?,
        id: object.cell.repr_hash(),
        message: object.object,
        proof: None,
        status: ton_block::MessageProcessingStatus::Finalized,
        transaction_id: None,
        transaction_now: None,
        ..Default::default()
    };

    let parsed = ton_block_json::db_serialize_message_ex(
        "id",
        &set,
        ton_block_json::SerializationMode::QServer,
    )
    .map_err(|err| Error::serialization_error(err, "message"))?;

    Ok(ResultOfParse {
        parsed: parsed.into(),
    })
}

/// Parses transaction boc into a JSON
///
/// JSON structure is compatible with GraphQL API transaction object
#[api_function]
pub async fn parse_transaction(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfParse,
) -> ClientResult<ResultOfParse> {
    let object =
        deserialize_object_from_boc::<ton_block::Transaction>(&context, &params.boc, "transaction").await?;

    let set = ton_block_json::TransactionSerializationSetEx {
        block_id: None,
        boc: &object.boc.bytes("transaction")?,
        id: &object.cell.repr_hash(),
        transaction: &object.object,
        proof: None,
        status: ton_block::TransactionProcessingStatus::Finalized,
        workchain_id: None
    };

    let parsed = ton_block_json::db_serialize_transaction_ex(
        "id",
        set,
        ton_block_json::SerializationMode::QServer,
    )
    .map_err(|err| Error::serialization_error(err, "transaction"))?;

    Ok(ResultOfParse {
        parsed: parsed.into(),
    })
}

/// Parses account boc into a JSON
///
/// JSON structure is compatible with GraphQL API account object
#[api_function]
pub async fn parse_account(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfParse,
) -> ClientResult<ResultOfParse> {
    let object = deserialize_object_from_boc::<ton_block::Account>(&context, &params.boc, "account").await?;

    let set = ton_block_json::AccountSerializationSet {
        boc: object.boc.bytes("account")?,
        proof: None,
        account: object.object,
        ..Default::default()
    };

    let parsed = ton_block_json::db_serialize_account_ex(
        "id",
        &set,
        ton_block_json::SerializationMode::QServer,
    )
    .map_err(|err| Error::serialization_error(err, "account"))?;

    Ok(ResultOfParse {
        parsed: parsed.into(),
    })
}

/// Parses block boc into a JSON
///
/// JSON structure is compatible with GraphQL API block object
#[api_function]
pub async fn parse_block(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfParse,
) -> ClientResult<ResultOfParse> {
    let object = deserialize_object_from_boc::<ton_block::Block>(&context, &params.boc, "block").await?;

    let set = ton_block_json::BlockSerializationSet {
        boc: object.boc.bytes("block")?,
        id: object.cell.repr_hash(),
        block: object.object,
        status: ton_block::BlockProcessingStatus::Finalized,
        ..Default::default()
    };

    let parsed = ton_block_json::db_serialize_block_ex(
        "id",
        &set,
        ton_block_json::SerializationMode::QServer,
    )
    .map_err(|err| Error::serialization_error(err, "block"))?;

    Ok(ResultOfParse {
        parsed: parsed.into(),
    })
}

/// Parses shardstate boc into a JSON
///
/// JSON structure is compatible with GraphQL API shardstate object
#[api_function]
pub async fn parse_shardstate(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfParseShardstate,
) -> ClientResult<ResultOfParse> {
    let object =
        deserialize_object_from_boc::<ton_block::ShardStateUnsplit>(&context, &params.boc, "shardstate").await?;

    let set = ton_block_json::ShardStateSerializationSet {
        boc: object.boc.bytes("shardstate")?,
        id: params.id,
        state: object.object,
        block_id: None,
        workchain_id: params.workchain_id,
        ..Default::default()
    };

    let parsed = ton_block_json::db_serialize_shard_state_ex(
        "id",
        &set,
        ton_block_json::SerializationMode::QServer,
    )
    .map_err(|err| Error::serialization_error(err, "shardstate"))?;

    Ok(ResultOfParse {
        parsed: parsed.into(),
    })
}

pub fn source_boc(parsed: &Value) -> ClientResult<String> {
    Ok(parsed["boc"]
        .as_str()
        .ok_or(Error::missing_source_boc())?
        .into())
}

pub fn required_boc(parsed: &Option<Value>) -> ClientResult<String> {
    if let Some(parsed) = parsed {
        Ok(source_boc(parsed)?)
    } else {
        Err(Error::missing_source_boc())
    }
}
