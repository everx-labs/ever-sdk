use crate::boc::Error;
use crate::error::ApiResult;
use ton_block::{Deserializable, Serializable};
use ton_types::deserialize_tree_of_cells;

pub(crate) fn get_boc_hash(boc: &[u8]) -> ApiResult<String> {
    let cells = deserialize_tree_of_cells(&mut boc.clone())
        .map_err(|err| crate::boc::Error::invalid_boc(err))?;
    let id: Vec<u8> = cells.repr_hash().as_slice()[..].into();
    Ok(hex::encode(&id))
}

pub(crate) fn deserialize_cell_from_base64(
    b64: &str,
    name: &str,
) -> ApiResult<(Vec<u8>, ton_types::Cell)> {
    let bytes = base64::decode(&b64)
        .map_err(|err| Error::invalid_boc(format!("error decode {} BOC base64: {}", name, err)))?;

    let cell = ton_types::cells_serialization::deserialize_tree_of_cells(&mut bytes.as_slice())
        .map_err(|err| {
            Error::invalid_boc(format!("{} BOC deserialization error: {}", name, err))
        })?;

    Ok((bytes, cell))
}

pub(crate) fn deserialize_object_from_cell<S: Deserializable>(
    cell: ton_types::Cell,
    name: &str,
) -> ApiResult<S> {
    S::construct_from(&mut cell.into())
        .map_err(|err| Error::invalid_boc(format!("cannot deserialize {} from BOC: {}", name, err)))
}

pub(crate) struct DeserializedObject<S: Deserializable> {
    pub boc: Vec<u8>,
    pub cell_hash: ton_types::UInt256,
    pub object: S,
}

pub(crate) fn deserialize_object_from_base64<S: Deserializable>(
    b64: &str,
    name: &str,
) -> ApiResult<DeserializedObject<S>> {
    let (bytes, cell) = deserialize_cell_from_base64(b64, name)?;

    let object = deserialize_object_from_cell(cell.clone(), name)?;

    Ok(DeserializedObject {
        boc: bytes,
        cell_hash: cell.repr_hash(),
        object,
    })
}

pub(crate) fn serialize_object_to_cell<S: Serializable>(
    object: &S,
    name: &str,
) -> ApiResult<ton_types::Cell> {
    Ok(object
        .serialize()
        .map_err(|err| Error::serialization_error(err, name))?)
}

pub(crate) fn serialize_cell_to_base64(cell: &ton_types::Cell, name: &str) -> ApiResult<String> {
    Ok(base64::encode(
        &ton_types::cells_serialization::serialize_toc(&cell)
            .map_err(|err| Error::serialization_error(err, name))?,
    ))
}

pub(crate) fn serialize_object_to_base64<S: Serializable>(
    object: &S,
    name: &str,
) -> ApiResult<String> {
    let cell = serialize_object_to_cell(object, name)?;
    Ok(serialize_cell_to_base64(&cell, name)?)
}
