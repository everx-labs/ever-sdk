use crate::*;
use serde_json::Value;
use tvm::block::{ Transaction as TvmTransaction };

#[cfg(test)]
#[path = "tests/test_check_proofs.rs"]
mod tests;

pub fn check_transaction(tr_val: Value) -> SdkResult<()> {
    // build transaction

    // build full transaction from BOC

    // check given transaction's JSON

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