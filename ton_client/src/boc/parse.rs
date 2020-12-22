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

use crate::boc::internal::deserialize_object_from_base64;
use crate::boc::Error;
use crate::client::ClientContext;
use crate::error::ClientResult;
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, ApiType)]
pub struct ParamsOfParse {
    /// BOC encoded as base64
    pub boc: String,
}

#[derive(Serialize, Deserialize, Clone, ApiType)]
pub struct ParamsOfParseShardstate {
    /// BOC encoded as base64
    pub boc: String,
    /// Shardstate identificator
    pub id: String,
    /// Workchain shardstate belongs to
    pub workchain_id: i32,
}

#[derive(Serialize, Deserialize, Clone, ApiType)]
pub struct ResultOfParse {
    /// JSON containing parsed BOC
    pub parsed: serde_json::Value,
}

/// Parses message boc into a JSON
///
/// JSON structure is compatible with GraphQL API message object
#[api_function]
pub fn parse_message(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfParse,
) -> ClientResult<ResultOfParse> {
    let object = deserialize_object_from_base64::<ton_block::Message>(&params.boc, "message")?;

    let set = ton_block_json::MessageSerializationSet {
        block_id: None,
        boc: object.boc,
        id: object.cell_hash,
        message: object.object,
        proof: None,
        status: ton_block::MessageProcessingStatus::Finalized,
        transaction_id: None,
        transaction_now: None,
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
pub fn parse_transaction(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfParse,
) -> ClientResult<ResultOfParse> {
    let object =
        deserialize_object_from_base64::<ton_block::Transaction>(&params.boc, "transaction")?;

    let set = ton_block_json::TransactionSerializationSetEx {
        block_id: None,
        boc: &object.boc,
        id: &object.cell_hash,
        transaction: &object.object,
        proof: None,
        status: ton_block::TransactionProcessingStatus::Finalized,
        workchain_id: None,
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
pub fn parse_account(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfParse,
) -> ClientResult<ResultOfParse> {
    let object = deserialize_object_from_base64::<ton_block::Account>(&params.boc, "account")?;

    let set = ton_block_json::AccountSerializationSet {
        boc: object.boc,
        proof: None,
        account: object.object,
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
pub fn parse_block(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfParse,
) -> ClientResult<ResultOfParse> {
    let object = deserialize_object_from_base64::<ton_block::Block>(&params.boc, "block")?;

    let set = ton_block_json::BlockSerializationSet {
        boc: object.boc,
        id: object.cell_hash,
        block: object.object,
        status: ton_block::BlockProcessingStatus::Finalized,
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
pub fn parse_shardstate(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfParseShardstate,
) -> ClientResult<ResultOfParse> {
    let object =
        deserialize_object_from_base64::<ton_block::ShardStateUnsplit>(&params.boc, "block")?;

    let set = ton_block_json::ShardStateSerializationSet {
        boc: object.boc,
        id: params.id,
        state: object.object,
        block_id: None,
        workchain_id: params.workchain_id,
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
