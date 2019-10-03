use crate::*;
use serde_json::Value;
use tvm::block::{ Transaction as TvmTransaction, TransactionProcessingStatus,
    deserialize_tree_of_cells_from_base64};

#[cfg(test)]
#[path = "tests/test_check_proofs.rs"]
mod tests;

pub fn check_transaction(tr_val: &Value) -> SdkResult<()> {

    // extracting boc and proof

    if !tr_val.is_object() {
        bail!(SdkErrorKind::InvalidData("Invalid transaction json"));
    }

    let tr_val_obj = tr_val.as_object().unwrap();
    let boc_val = tr_val_obj.get("boc");
    let proof_val = tr_val_obj.get("proof");

    if boc_val.is_none() || proof_val.is_none() ||
        boc_val.unwrap().is_null() || proof_val.unwrap().is_null() {
        if let Some(status_val) = tr_val_obj.get("status") {
            let status: TransactionProcessingStatus = serde_json::from_value(status_val)
                .map_err(|err| {
                    SdkErrorKind::InvalidData(format!("error parsing transaction's status: {}", err))
                })?;
            if status == TransactionProcessingStatus::Finalized 
                || status == TransactionProcessingStatus::Proposed {
                bail!(SdkErrorKind::InvalidData(
                    "Finalized or Proposed transactions must contain both proof and boc fields"));
            } else {
                return Ok(tr);
            }
        } else {
            bail!(SdkErrorKind::InvalidData(
                "Transaction JSON must contain both proof and boc fields or not Finalized or Proposed status"));
        }
    }

    // build full transaction from BOC, extract boc & proof
    let mut boc = deserialize_tree_of_cells_from_base64(&str::from(boc_val.unwrap()))
        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing boc: {}", err)))?;
    let mut proof = deserialize_tree_of_cells_from_base64(&str::from(proof_val.unwrap()))
        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing boc: {}", err)))?;
    let mut full_tr = TvmTransaction::construct_from(&mut tr.boc.unwrap().clone());


    // check given transaction's JSON
    let complete_json = serde_json::to_value(full_tr)
        .map_err(|err| {
            SdkErrorKind::InvalidData(format!("error serializing (to json) full transaction: {}", err))
        })?;
    check_incomplete_json()

    // check merkle proof





    unimplemented!()
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
                bail!(SdkErrorKind::WrongJson);
            }
        }
    }
    ok!()
}