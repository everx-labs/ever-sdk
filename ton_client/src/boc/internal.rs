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

use crate::ClientContext;
use crate::boc::{BocCacheType, Error};
use crate::error::ClientResult;
use std::io::Cursor;
#[allow(unused_imports)]
use std::str::FromStr;
use ton_block::{Deserializable, Serializable};
use ton_types::{UInt256, deserialize_tree_of_cells};

pub fn get_boc_hash(boc: &[u8]) -> ClientResult<String> {
    let cells = deserialize_tree_of_cells(&mut Cursor::new(boc))
        .map_err(|err| Error::invalid_boc(err))?;
    let id: Vec<u8> = cells.repr_hash().as_slice()[..].into();
    Ok(hex::encode(&id))
}

pub fn deserialize_cell_from_base64(
    b64: &str,
    name: &str,
) -> ClientResult<(Vec<u8>, ton_types::Cell)> {
    let bytes = base64::decode(&b64)
        .map_err(|err| Error::invalid_boc(format!("error decode {} BOC base64: {}", name, err)))?;

    let cell = deserialize_tree_of_cells(&mut Cursor::new(&bytes))
        .map_err(|err| {
            Error::invalid_boc(format!("{} BOC deserialization error: {}", name, err))
        })?;

    Ok((bytes, cell))
}

pub fn deserialize_object_from_cell<S: Deserializable>(
    cell: ton_types::Cell,
    name: &str,
) -> ClientResult<S> {
    let tip = match name {
        "message" => "Please check that you have specified the message's BOC, not body, as a parameter.",
        _ => "",
    };
    let tip_full = if tip.len() > 0 {
        format!(".\nTip: {}", tip)
    } else {
        "".to_string()
    };
    S::construct_from_cell(cell)
        .map_err(|err|
            Error::invalid_boc(
                format!("cannot deserialize {} from BOC: {}{}", name, err, tip_full)
            )
        )
}

#[derive(Clone)]
pub enum DeserializedBoc {
    Cell(ton_types::Cell),
    Bytes(Vec<u8>),
}

impl DeserializedBoc {
    pub fn bytes(self, name: &str) -> ClientResult<Vec<u8>> {
        match self {
            DeserializedBoc::Bytes(vec) => Ok(vec),
            DeserializedBoc::Cell(cell) => serialize_cell_to_bytes(&cell, name)
        }
    }
}

#[derive(Clone)]
pub struct DeserializedObject<S: Deserializable> {
    pub boc: DeserializedBoc,
    pub cell: ton_types::Cell,
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
) -> ClientResult<ton_types::Cell> {
    Ok(object
        .serialize()
        .map_err(|err| Error::serialization_error(err, name))?)
}

pub fn serialize_cell_to_bytes(cell: &ton_types::Cell, name: &str) -> ClientResult<Vec<u8>> {
    ton_types::cells_serialization::serialize_toc(&cell)
        .map_err(|err| Error::serialization_error(err, name))
}

pub fn serialize_cell_to_base64(cell: &ton_types::Cell, name: &str) -> ClientResult<String> {
    Ok(base64::encode(&serialize_cell_to_bytes(cell, name)?))
}

pub fn serialize_object_to_base64<S: Serializable>(
    object: &S,
    name: &str,
) -> ClientResult<String> {
    let cell = serialize_object_to_cell(object, name)?;
    Ok(serialize_cell_to_base64(&cell, name)?)
}

pub async fn deserialize_cell_from_boc(
    context: &ClientContext, boc: &str, name: &str
) -> ClientResult<(DeserializedBoc, ton_types::Cell)> {
    if boc.starts_with("*") {
        let hash = UInt256::from_str(&boc[1..])
            .map_err(|err| Error::invalid_boc(
                format!("BOC start with `*` but contains invalid hash: {}", err)
            ))?;

        let cell = context.bocs
            .get(&hash)
            .await
            .ok_or(Error::boc_ref_not_found(boc))?;
        Ok((DeserializedBoc::Cell(cell.clone()), cell))
    } else {
        deserialize_cell_from_base64(boc, name)
            .map(|(bytes, cell)| (DeserializedBoc::Bytes(bytes), cell))
    }
}

pub async fn deserialize_object_from_boc<S: Deserializable>(
    context: &ClientContext, boc: &str, name: &str,
) -> ClientResult<DeserializedObject<S>> {
    let (boc, cell) = deserialize_cell_from_boc(context, boc, name).await?;

    let object = deserialize_object_from_cell(cell.clone(), name)?;

    Ok(DeserializedObject {
        boc,
        cell,
        object,
    })
}

pub fn deserialize_object_from_boc_bin<S: Deserializable>(
    boc: &[u8],
) -> ClientResult<(S, UInt256)> {
    let cell = deserialize_tree_of_cells(&mut Cursor::new(boc))
        .map_err(|err| Error::invalid_boc(err))?;
    let root_hash = cell.repr_hash();
    let object = S::construct_from_cell(cell)
        .map_err(|err| Error::invalid_boc(err))?;

    Ok((object, root_hash))
}

pub async fn serialize_cell_to_boc(
    context: &ClientContext, cell: ton_types::Cell, name: &str, boc_cache: Option<BocCacheType>,
) -> ClientResult<String> {
    if let Some(cache_type) = boc_cache {
        context.bocs.add(cache_type, cell, None)
            .await
            .map(|hash| format!("*{:x}", hash))
    } else {
        serialize_cell_to_base64(&cell, name)
    }
}

pub async fn serialize_object_to_boc<S: Serializable>(
    context: &ClientContext, object: &S,name: &str, boc_cache: Option<BocCacheType>,
) -> ClientResult<String> {
    let cell = serialize_object_to_cell(object, name)?;
    serialize_cell_to_boc(context, cell, name, boc_cache).await
}
