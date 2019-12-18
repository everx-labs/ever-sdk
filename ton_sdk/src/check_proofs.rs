use crate::*;
use std::str::FromStr;
use serde_json::Value;
use ton_block::{ Transaction as TvmTransaction, Message as TvmMessage, 
    TransactionProcessingStatus, Deserializable, Account as TvmAccount, check_account_proof,
    check_transaction_proof, check_message_proof, MerkleProof, MessageProcessingStatus,
    BlockSeqNoAndShard };
use ton_types::cells_serialization::deserialize_tree_of_cells;
use ton_vm::types::UInt256;

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
        if let Some(status_num) = status_val.as_u64() {
            status = match status_num {
                1 => TransactionProcessingStatus::Preliminary,
                2 => TransactionProcessingStatus::Proposed,
                3 => TransactionProcessingStatus::Finalized,
                4 => TransactionProcessingStatus::Refused,
                _ => bail!(SdkErrorKind::InvalidData("Error parsing message's status: unknown value".into()))
            };
        } else {
            bail!(SdkErrorKind::InvalidData("Status field must be a number".into()));
        }
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
                return Ok(());
            }
    }

    let block_id_str = mandatory_values.remove(2).unwrap().as_str().unwrap();
    let proof_str = mandatory_values.remove(1).unwrap().as_str().unwrap();
    let boc_str = mandatory_values.remove(0).unwrap().as_str().unwrap();

    // parse boc, proof and block_id

    let proof_bytes = base64::decode(proof_str)
        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing proof: {}", err)))?;
    let proof = deserialize_tree_of_cells(&mut std::io::Cursor::new(&proof_bytes))
        .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize proof: {}", err)))?;

    let boc_bytes = base64::decode(boc_str)
        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing boc: {}", err)))?;
    let boc = deserialize_tree_of_cells(&mut std::io::Cursor::new(&boc_bytes))
        .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize boc: {}", err)))?;

    let block_id = UInt256::from_str(block_id_str)
        .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize block id: {}", err)))?
        .into();

    // build full transaction from BOC
    let full_tr: TvmTransaction = TvmTransaction::construct_from(&mut boc.clone().into())?;

    // and proof
    let proof: MerkleProof = MerkleProof::construct_from(&mut proof.into())?;

    // check merkle proof
    check_transaction_proof(&proof, &full_tr, &block_id)?;

    // check given transaction's JSON
    let ser_set = ton_block_json::TransactionSerializationSet {
        transaction: full_tr,
        id: boc.repr_hash(),
        status,
        block_id: Some(block_id),
        boc: vec!(),  // There's no point in checking these fields because it is got
        proof: None,  // from checked jsons (it is skipped in check_incomplete_json)
    };

    let complete_json = json!(Value::from(ton_block_json::db_serialize_transaction("id", &ser_set)));
    check_incomplete_json(tr_val, &complete_json)?;

    Ok(())
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

    // extracting "special" values (which absent in blockchain struct)

    if !msg_val.is_object() {
        bail!(SdkErrorKind::InvalidData("Invalid message json".into()));
    }

    let msg_val_obj = msg_val.as_object().unwrap();
    let mut mandatory_values: Vec<Option<&Value>> = vec!["boc", "proof", "block_id"]
        .iter().map(|n| msg_val_obj.get(*n)).collect();

    let status: MessageProcessingStatus;
    if let Some(status_val) = msg_val_obj.get("status") {
        if let Some(status_num) = status_val.as_u64() {
            status = match status_num {
                1 => MessageProcessingStatus::Queued,
                2 => MessageProcessingStatus::Processing,
                3 => MessageProcessingStatus::Preliminary,
                4 => MessageProcessingStatus::Proposed,
                5 => MessageProcessingStatus::Finalized,
                6 => MessageProcessingStatus::Refused,
                7 => MessageProcessingStatus::Transiting,
                _ => bail!(SdkErrorKind::InvalidData("Error parsing message's status: unknown value".into()))
            };
        } else {
            bail!(SdkErrorKind::InvalidData("Status field must be a number".into()));
        }
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
                return Ok(());
            }
    }

    let block_id_str = mandatory_values.remove(2).unwrap().as_str().unwrap();
    let proof_str = mandatory_values.remove(1).unwrap().as_str().unwrap();
    let boc_str = mandatory_values.remove(0).unwrap().as_str().unwrap();

    let transaction_id_str = msg_val_obj
        .get("transaction_id")
        .and_then(|tv| tv.as_str());

    // parse "special" values

    let proof_bytes = base64::decode(proof_str)
        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing proof: {}", err)))?;
    let proof = deserialize_tree_of_cells(&mut std::io::Cursor::new(&proof_bytes))
        .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize proof: {}", err)))?;

    let boc_bytes = base64::decode(boc_str)
        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing boc: {}", err)))?;
    let boc = deserialize_tree_of_cells(&mut std::io::Cursor::new(&boc_bytes))
        .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize boc: {}", err)))?;

    let block_id = UInt256::from_str(block_id_str)
        .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize block id: {}", err)))?
        .into();

    let transaction_id = if let Some(id_str) = transaction_id_str {
        Some(
            UInt256::from_str(id_str)
                .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize transaction id: {}", err)))?
        )
    } else {
        None
    };

    // build full message from BOC
    let full_msg: TvmMessage = TvmMessage::construct_from(&mut boc.clone().into())?;

    // and proof
    let proof: MerkleProof = MerkleProof::construct_from(&mut proof.into())?;

    // check merkle proof
    check_message_proof(&proof, &full_msg, &block_id)?;

    // check given message's JSON
    let ser_set = ton_block_json::MessageSerializationSet {
        message: full_msg,
        id: boc.repr_hash(),
        block_id: Some(block_id),
        transaction_id,
        status,
        boc: vec!(),  // There's no point in checking these fields because it is got
        proof: None,  // from checked jsons (it is skipped in check_incomplete_json)
    };

    let complete_json = json!(Value::from(ton_block_json::db_serialize_message("id", &ser_set)));
    check_incomplete_json(msg_val, &complete_json)?;

    Ok(())
}


/// Checks merkle proof of account. Takes serde_json::Value with account struct.
/// Value must contain `addr.AddrStd.address`, `boc` and `proof` fields.
/// The next checks are performed:
/// * check if all account's fields from value 
/// is corresponds to values in account constructed from boc;
/// * check if account merkle proof is correct (account is a part of shard state with given (in proof) root hash).
/// Returns shard state's root hash and correspond block's info for future checks.
#[allow(dead_code)]
pub fn check_account(acc_val: &Value) -> SdkResult<(UInt256, BlockSeqNoAndShard)> {

    // extracting boc and proof

    if !acc_val.is_object() {
        bail!(SdkErrorKind::InvalidData("Invalid account json".into()));
    }

    let acc_val_obj = acc_val.as_object().unwrap();
    let mut mandatory_values: Vec<Option<&Value>> = vec!["boc", "proof"]
        .iter().map(|n| acc_val_obj.get(*n)).collect();

    if mandatory_values.iter().any(
        |mv| mv.is_none() || mv.unwrap().is_null() || !mv.unwrap().is_string()) {

        bail!(SdkErrorKind::InvalidData(
            "Account must contain both proof and boc fields".into()));
    }

    let proof_str = mandatory_values.remove(1).unwrap().as_str().unwrap();
    let boc_str = mandatory_values.remove(0).unwrap().as_str().unwrap();

    // parse boc and proof

    let proof_bytes = base64::decode(proof_str)
        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing proof: {}", err)))?;
    let proof = deserialize_tree_of_cells(&mut std::io::Cursor::new(&proof_bytes))
        .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize proof: {}", err)))?;

    let boc_bytes = base64::decode(boc_str)
        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing boc: {}", err)))?;
    let boc = deserialize_tree_of_cells(&mut std::io::Cursor::new(&boc_bytes))
        .map_err(|err| SdkErrorKind::InvalidData(format!("error deserialize boc: {}", err)))?;


    // build full account from BOC
    let full_acc: TvmAccount = TvmAccount::construct_from(&mut boc.into())?;

    // and proof
    let proof: MerkleProof = MerkleProof::construct_from(&mut proof.into())?;

    // check merkle proof
    let block_inf = check_account_proof(&proof, &full_acc)?;

    // check given account's JSON
    let ser_set = ton_block_json::AccountSerializationSet {
        account: full_acc,
        boc: vec!(),  // There's no point in checking these fields because it is got
        proof: None,  // from checked jsons (it is skipped in check_incomplete_json)
    };

    let complete_json = json!(Value::from(ton_block_json::db_serialize_account("id", &ser_set)));
    check_incomplete_json(acc_val, &complete_json)?;

    Ok((proof.hash.clone(), block_inf))
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
    Ok(())
}