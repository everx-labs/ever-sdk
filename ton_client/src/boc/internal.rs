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

use crate::boc::{BocCacheType, Error};
use crate::error::ClientResult;
use crate::ClientContext;
#[allow(unused_imports)]
use std::str::FromStr;
use ever_block::{Deserializable, Serializable};
use ever_block::UInt256;

pub(crate) fn get_boc_hash(boc: &[u8]) -> ClientResult<String> {
    let cells =
        ever_block::boc::read_single_root_boc(&boc).map_err(|err| Error::invalid_boc(err))?;
    let id: Vec<u8> = cells.repr_hash().as_slice()[..].into();
    Ok(hex::encode(&id))
}

pub fn deserialize_cell_from_base64(
    b64: &str,
    name: &str,
) -> ClientResult<(Vec<u8>, ever_block::Cell)> {
    let bytes = base64::decode(&b64)
        .map_err(|err| Error::invalid_boc(format!("error decode {} BOC base64: {}", name, err)))?;

    let cell = ever_block::boc::read_single_root_boc(&bytes).map_err(|err| {
        Error::invalid_boc(format!("{} BOC deserialization error: {}", name, err))
    })?;

    Ok((bytes, cell))
}

pub fn deserialize_object_from_cell<S: Deserializable>(
    cell: ever_block::Cell,
    name: &str,
) -> ClientResult<S> {
    let tip = match name {
        "message" => {
            "Please check that you have specified the message's BOC, not body, as a parameter."
        }
        _ => "",
    };
    let tip_full = if tip.len() > 0 {
        format!(".\nTip: {}", tip)
    } else {
        "".to_string()
    };
    S::construct_from_cell(cell).map_err(|err| {
        Error::invalid_boc(format!(
            "cannot deserialize {} from BOC: {}{}",
            name, err, tip_full
        ))
    })
}

#[derive(Clone)]
pub enum DeserializedBoc {
    Cell(ever_block::Cell),
    Bytes(Vec<u8>),
}

impl DeserializedBoc {
    pub fn bytes(self, name: &str) -> ClientResult<Vec<u8>> {
        match self {
            DeserializedBoc::Bytes(vec) => Ok(vec),
            DeserializedBoc::Cell(cell) => serialize_cell_to_bytes(&cell, name),
        }
    }
}

#[derive(Clone)]
pub struct DeserializedObject<S: Deserializable> {
    pub boc: DeserializedBoc,
    pub cell: ever_block::Cell,
    pub object: S,
}

pub fn deserialize_object_from_base64<S: Deserializable>(
    b64: &str,
    name: &str,
) -> ClientResult<DeserializedObject<S>> {
    let (bytes, cell) = deserialize_cell_from_base64(b64, name)?;
    let object = deserialize_object_from_cell(cell.clone(), name)?;

    Ok(DeserializedObject {
        boc: DeserializedBoc::Bytes(bytes),
        cell,
        object,
    })
}

pub fn serialize_object_to_cell<S: Serializable>(
    object: &S,
    name: &str,
) -> ClientResult<ever_block::Cell> {
    Ok(object
        .serialize()
        .map_err(|err| Error::serialization_error(err, name))?)
}

pub fn serialize_cell_to_bytes(cell: &ever_block::Cell, name: &str) -> ClientResult<Vec<u8>> {
    ever_block::boc::write_boc(&cell)
        .map_err(|err| Error::serialization_error(err, name))
}

pub fn serialize_cell_to_base64(cell: &ever_block::Cell, name: &str) -> ClientResult<String> {
    Ok(base64::encode(&serialize_cell_to_bytes(cell, name)?))
}

pub fn serialize_object_to_base64<S: Serializable>(
    object: &S,
    name: &str,
) -> ClientResult<String> {
    let cell = serialize_object_to_cell(object, name)?;
    Ok(serialize_cell_to_base64(&cell, name)?)
}

pub fn deserialize_cell_from_boc(
    context: &ClientContext, boc: &str, name: &str
) -> ClientResult<(DeserializedBoc, ever_block::Cell)> {
    context.bocs.deserialize_cell(boc, name)
}

pub fn deserialize_object_from_boc<S: Deserializable>(
    context: &ClientContext, boc: &str, name: &str,
) -> ClientResult<DeserializedObject<S>> {
    let (boc, cell) = deserialize_cell_from_boc(context, boc, name)?;

    let object = deserialize_object_from_cell(cell.clone(), name)?;

    Ok(DeserializedObject { boc, cell, object })
}

pub fn deserialize_object_from_boc_bin<S: Deserializable>(
    boc: &[u8],
) -> ClientResult<(S, UInt256)> {
    let cell =
        ever_block::boc::read_single_root_boc(&boc).map_err(|err| Error::invalid_boc(err))?;
    let root_hash = cell.repr_hash();
    let object = S::construct_from_cell(cell).map_err(|err| Error::invalid_boc(err))?;

    Ok((object, root_hash))
}

pub fn serialize_cell_to_boc(
    context: &ClientContext, cell: ever_block::Cell, name: &str, boc_cache: Option<BocCacheType>,
) -> ClientResult<String> {
    if let Some(cache_type) = boc_cache {
        context
            .bocs
            .add(cache_type, cell, None)
            .map(|hash| format!("*{:x}", hash))
    } else {
        serialize_cell_to_base64(&cell, name)
    }
}

pub fn serialize_object_to_boc<S: Serializable>(
    context: &ClientContext, object: &S,name: &str, boc_cache: Option<BocCacheType>,
) -> ClientResult<String> {
    let cell = serialize_object_to_cell(object, name)?;
    serialize_cell_to_boc(context, cell, name, boc_cache)
}
