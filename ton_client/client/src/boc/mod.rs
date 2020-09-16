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

use crate::dispatch::DispatchTable;
use crate::client::ClientContext;
use crate::error::{ApiResult};
use ton_block::Deserializable;
use errors::Error;

mod errors;

// TODO: uncomment when module will be ready
//mod cell;

#[cfg(test)]
mod tests;

#[derive(Serialize, Deserialize, Clone, TypeInfo)]
pub struct ParamsOfParse {
    /// BOC encoded as base64
    pub boc: String,
}

#[derive(Serialize, Deserialize, Clone, TypeInfo)]
pub struct ResultOfParse {
    /// JSON containing parsed BOC
    pub parsed: serde_json::Value,
}

fn deserialize_cell_from_base64(b64: &str) -> ApiResult<(Vec<u8>, ton_types::Cell)>
{
    let bytes = base64::decode(&b64)
        .map_err(|err| Error::invalid_boc(format!("error decode base64: {}", err)))?;

    let cell = ton_types::cells_serialization::deserialize_tree_of_cells(&mut bytes.as_slice())
        .map_err(|err| Error::invalid_boc(format!("BOC deserialization error: {}", err)))?;

    Ok((bytes, cell))
}

struct DeserializedObject<S: Deserializable> {
    boc: Vec<u8>,
    cell_hash: ton_types::UInt256,
    object: S,
}

fn deserialize_object_from_base64<S: Deserializable>(b64: &str, name: &str) -> ApiResult<DeserializedObject<S>>
{
    let (bytes, cell) = deserialize_cell_from_base64(b64)?;

    let object = S::construct_from(&mut cell.clone().into())
        .map_err(|err| Error::invalid_boc(format!("cannot deserialize {} from BOC: {}", name, err)))?;

    Ok(DeserializedObject {
        boc: bytes,
        cell_hash: cell.repr_hash(),
        object
    })
}

pub fn parse_message(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfParse,
) -> ApiResult<ResultOfParse> {
    let object = 
        deserialize_object_from_base64::<ton_block::Message>(&params.boc, "message")?;

    let set = ton_block_json::MessageSerializationSet {
        block_id: None,
        boc: object.boc,
        id: object.cell_hash,
        message: object.object,
        proof: None,
        status: ton_block::MessageProcessingStatus::Finalized,
        transaction_id: None,
        transaction_now: None
    };

    let parsed = ton_block_json::db_serialize_message_ex(
        "id",
        &set,
        ton_block_json::SerializationMode::QServer
    ).map_err(|err| Error::serialization_error(err, "message"))?;

    Ok(ResultOfParse {
        parsed: parsed.into()
    })
}

pub fn parse_transaction(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfParse,
) -> ApiResult<ResultOfParse> {
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
        ton_block_json::SerializationMode::QServer
    ).map_err(|err| Error::serialization_error(err, "transaction"))?;

    Ok(ResultOfParse {
        parsed: parsed.into()
    })
}

pub fn parse_account(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfParse,
) -> ApiResult<ResultOfParse> {
    let object = 
        deserialize_object_from_base64::<ton_block::Account>(&params.boc, "account")?;

    let set = ton_block_json::AccountSerializationSet {
        boc: object.boc,
        proof: None,
        account: object.object,
    };

    let parsed = ton_block_json::db_serialize_account_ex(
        "id",
        &set,
        ton_block_json::SerializationMode::QServer
    ).map_err(|err| Error::serialization_error(err, "account"))?;

    Ok(ResultOfParse {
        parsed: parsed.into()
    })
}

pub fn parse_block(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfParse,
) -> ApiResult<ResultOfParse> {
    let object = 
        deserialize_object_from_base64::<ton_block::Block>(&params.boc, "block")?;

    let set = ton_block_json::BlockSerializationSet {
        boc: object.boc,
        id: object.cell_hash,
        block: object.object,
        status: ton_block::BlockProcessingStatus::Finalized,
    };

    let parsed = ton_block_json::db_serialize_block_ex(
        "id",
        &set,
        ton_block_json::SerializationMode::QServer
    ).map_err(|err| Error::serialization_error(err, "block"))?;

    Ok(ResultOfParse {
        parsed: parsed.into()
    })
}

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.call("boc.parse_message", parse_message);
    handlers.call("boc.parse_transaction", parse_transaction);
    handlers.call("boc.parse_account", parse_account);
    handlers.call("boc.parse_block", parse_block);
}
