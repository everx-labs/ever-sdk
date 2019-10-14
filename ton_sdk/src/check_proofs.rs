use crate::*;
use std::str::FromStr;
use serde_json::Value;
use tvm::block::{ Transaction as TvmTransaction, Message as TvmMessage, TransactionProcessingStatus, Deserializable, 
    check_transaction_proof, check_message_proof, MerkleProof, MessageProcessingStatus };
use tvm::cells_serialization::deserialize_tree_of_cells;
use tvm::types::UInt256;

#[cfg(test)]
#[path = "tests/test_check_proofs.rs"]
mod tests;

/// Checks merkle proof of transaction. Takes serde_json::Value with transaction struct.
/// The value must contain `status` field.
/// * if the status is `Finalized` or `Proposed` the value
/// must contain `boc`, `proof` and `block_id` field also;
/// * if the status is not `Finalized` or `Proposed` 
/// and any of `boc`, `proof` and `block_id` fields is absent - 
/// function just return Ok() without any checks.
/// Only if the value contains `boc`, `proof` and `block_id` field,
/// the next checks are performed:
/// * check if all transaction's fields from value 
/// is corresponds to values in transaction constructed from boc;
/// * check if transaction merkle proof is correct (transaction is a part of block with given id).
#[allow(dead_code)]
pub fn check_transaction(tr_val: &Value) -> SdkResult<()> {

    // extracting boc, proof and block_id strings

    if !tr_val.is_object() {
        bail!(SdkErrorKind::InvalidData("Invalid transaction json".into()));
    }

    let tr_val_obj = tr_val.as_object().unwrap();
    let mut mandatory_values: Vec<Option<&Value>> = vec!["boc", "proof", "block_id"]
        .iter().map(|n| tr_val_obj.get(*n)).collect();

    let status: TransactionProcessingStatus;
    if let Some(status_val) = tr_val_obj.get("status") {
        status = serde_json::from_value(status_val.clone())
            .map_err(|err| {
                SdkErrorKind::InvalidData(format!("error parsing transaction's status: {}", err))
            })?;
    } else {
        bail!(SdkErrorKind::InvalidData(
            "Transaction JSON must contain status field".into()));
    }

    if mandatory_values.iter().any(
        |mv| mv.is_none() || mv.unwrap().is_null() || !mv.unwrap().is_string()) {

            if status == TransactionProcessingStatus::Finalized 
                || status == TransactionProcessingStatus::Proposed {
                bail!(SdkErrorKind::InvalidData(
                    "Finalized or Proposed transactions must contain both proof and boc fields".into()));
            } else {
                return ok!();
            }
    }

    let block_id_str = mandatory_values.remove(2).unwrap().as_str().unwrap();
    let proof_str = mandatory_values.remove(1).unwrap().as_str().unwrap();
    let boc_str = mandatory_values.remove(0).unwrap().as_str().unwrap();

    // parse boc, proof and block_id

    let proof_bytes = base64::decode(proof_str)
        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing proof: {}", err)))?;
    let proof = deserialize_tree_of_cells(&mut std::io::Cursor::new(proof_bytes))
        .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize proof: {}", err)))?;

    let boc_bytes = base64::decode(boc_str)
        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing boc: {}", err)))?;
    let boc = deserialize_tree_of_cells(&mut std::io::Cursor::new(boc_bytes))
        .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize boc: {}", err)))?;

    let block_id = Some(UInt256::from_str(block_id_str)
        .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize block id: {}", err)))?
        .into());

    // build full transaction from BOC
    let mut full_tr: TvmTransaction = TvmTransaction::construct_from(&mut boc.into())?;
    full_tr.block_id = block_id;
    full_tr.status = status;

    // and proof
    let proof: MerkleProof = MerkleProof::construct_from(&mut proof.into())?;

    // check merkle proof
    check_transaction_proof(&proof, &full_tr)?;

    // check given transaction's JSON
    let complete_json = serde_json::to_value(full_tr)
        .map_err(|err| {
            SdkErrorKind::InvalidData(format!("error serializing (to json) full transaction: {}", err))
        })?;
    check_incomplete_json(tr_val, &complete_json)?;

    ok!()
}

/// Checks merkle proof of message. Takes serde_json::Value with message struct.
/// The value must contain `status` field.
/// * if the status is `Finalized` or `Proposed` the value
/// must contain `boc`, `proof` and `block_id` field also;
/// * if the status is not `Finalized` or `Proposed` 
/// and any of `boc`, `proof` and `block_id` fields is absent - 
/// function just return Ok() without any checks.
/// Only if the value contains `boc`, `proof` and `block_id` field,
/// the next checks are performed:
/// * check if all message's fields from value 
/// is corresponds to values in message constructed from boc;
/// * check if message merkle proof is correct (message is a part of block with given id).
#[allow(dead_code)]
pub fn check_message(msg_val: &Value) -> SdkResult<()> {

    // extracting boc, proof and block_id strings

    if !msg_val.is_object() {
        bail!(SdkErrorKind::InvalidData("Invalid message json".into()));
    }

    let msg_val_obj = msg_val.as_object().unwrap();
    let mut mandatory_values: Vec<Option<&Value>> = vec!["boc", "proof", "block_id"]
        .iter().map(|n| msg_val_obj.get(*n)).collect();

    let status: MessageProcessingStatus;
    if let Some(status_val) = msg_val_obj.get("status") {
        status = serde_json::from_value(status_val.clone())
            .map_err(|err| {
                SdkErrorKind::InvalidData(format!("error parsing message's status: {}", err))
            })?;
    } else {
        bail!(SdkErrorKind::InvalidData(
            "Message JSON must contain status field".into()));
    }

    if mandatory_values.iter().any(
        |mv| mv.is_none() || mv.unwrap().is_null() || !mv.unwrap().is_string()) {

            if status == MessageProcessingStatus::Finalized 
                || status == MessageProcessingStatus::Proposed {
                bail!(SdkErrorKind::InvalidData(
                    "Finalized or Proposed messages must contain both proof and boc fields".into()));
            } else {
                return ok!();
            }
    }

    let block_id_str = mandatory_values.remove(2).unwrap().as_str().unwrap();
    let proof_str = mandatory_values.remove(1).unwrap().as_str().unwrap();
    let boc_str = mandatory_values.remove(0).unwrap().as_str().unwrap();

    // parse boc, proof and block_id

    let proof_bytes = base64::decode(proof_str)
        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing proof: {}", err)))?;
    let proof = deserialize_tree_of_cells(&mut std::io::Cursor::new(proof_bytes))
        .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize proof: {}", err)))?;

    let boc_bytes = base64::decode(boc_str)
        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing boc: {}", err)))?;
    let boc = deserialize_tree_of_cells(&mut std::io::Cursor::new(boc_bytes))
        .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize boc: {}", err)))?;

    let block_id = Some(UInt256::from_str(block_id_str)
        .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize block id: {}", err)))?
        .into());

    // build full message from BOC
    let mut full_msg: TvmMessage = TvmMessage::construct_from(&mut boc.into())?;
    full_msg.block_id = block_id;
    full_msg.status = status;

    // and proof
    let proof: MerkleProof = MerkleProof::construct_from(&mut proof.into())?;

    // check merkle proof
    check_message_proof(&proof, &full_msg)?;

    // check given message's JSON
    let complete_json = serde_json::to_value(full_msg)
        .map_err(|err| {
            SdkErrorKind::InvalidData(format!("error serializing (to json) full message: {}", err))
        })?;
    check_incomplete_json(msg_val, &complete_json)?;

    ok!()
}

fn check_incomplete_json(incomplete: &Value, complete: &Value) -> SdkResult<()> {
    check_incomplete_json_internal(incomplete, complete, true)
}

fn check_incomplete_json_internal(incomplete: &Value, complete: &Value, root: bool) -> SdkResult<()> {
    // All fields from incomplete_json must exists in complete_json
    match incomplete {
        Value::Array(values) => {
            if !complete.is_array() {
                bail!(SdkErrorKind::WrongJson);
            }
            // Arrays must contain same set of values
            let complete_values = complete.as_array().unwrap();
            if values.len() != complete_values.len() {
                bail!(SdkErrorKind::WrongJson);
            }
            for (value, complete_value) in values.iter().zip(complete_values.iter()) {
                check_incomplete_json_internal(value, complete_value, false)?;
            }
        },
        Value::Object(map) => {
            if !complete.is_object() {
                bail!(SdkErrorKind::WrongJson);
            }
            let complete_map = complete.as_object().unwrap();
            // All values from incomplete_json must exists in complete_json
            for (key, value) in map.iter() {
                // skip special values in root json's level
                if !root || (key != "boc" && key != "proof") {
                    if let Some(complete_value) = complete_map.get(key) {
                        check_incomplete_json_internal(value, complete_value, false)?;
                    } else if root && key == "_id" {
                        if let Some(complete_value) = complete_map.get("id") {
                            check_incomplete_json_internal(value, complete_value, false)?;
                        } else {
                            bail!(SdkErrorKind::WrongJson);
                        }
                    } else {
                        bail!(SdkErrorKind::WrongJson);
                    }
                }
            }
        },
        Value::Null => (),
        incomplete => {
            if incomplete != complete {
                dbg!(incomplete);
                dbg!(complete);
                bail!(SdkErrorKind::WrongJson);
            }
        }
    }
    ok!()
}