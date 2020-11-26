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
 *
 */

use crate::boc::internal::serialize_cell_to_base64;
use crate::error::ClientResult;
use crate::tvm::Error;
use core::result::Result::{Err, Ok};
use serde_json::Value;
use std::ops::Deref;
use std::slice::Iter;
use std::sync::Arc;
use ton_vm::stack::integer::IntegerData;
use ton_vm::stack::StackItem;

pub fn serialize_items(items: Iter<StackItem>) -> ClientResult<Value> {
    let mut values = Vec::<Value>::new();
    for item in items {
        values.push(serialize_item(item)?)
    }
    Ok(Value::Array(values))
}

pub fn deserialize_items(values: Iter<Value>) -> ClientResult<Vec<StackItem>> {
    let mut items = Vec::<StackItem>::new();
    for value in values {
        items.push(deserialize_item(value)?)
    }
    Ok(items)
}

fn serialize_integer_data(data: &ton_vm::stack::integer::IntegerData) -> String {
    let hex = data.to_str_radix(16);
    // all negative nubers and positive numbers less than u128::MAX are encoded as decimal
    if hex.starts_with("-") || hex.len() <= 32 {
        data.to_str_radix(10)
    } else {
        // positive numbers between u128::MAX and u256::MAX are padded to 64 hex symbols
        if hex.len() <= 64 {
            format!("0x{:0>64}", hex)
        } else {
            // positive numbers between u256::MAX and u512::MAX are padded to 128 symbols
            // positive numbers above u512::MAX are not padded
            format!("0x{:0>128}", hex)
        }
    }
}

pub fn serialize_item(item: &StackItem) -> ClientResult<Value> {
    Ok(match item {
        StackItem::None => Value::Null,
        StackItem::Integer(value) => Value::String(serialize_integer_data(value)),
        StackItem::Tuple(items) => serialize_items(items.iter())?,
        StackItem::Builder(value) => json!({
            "type": "Builder",
            "value": serialize_cell_to_base64(&value.deref().into(), "stack item `Builder`")
        }),
        StackItem::Slice(value) => json!({
            "type": "Slice",
            "value": serialize_cell_to_base64(&value.into_cell(), "stack item `Slice`")
        }),
        StackItem::Cell(value) => json!({
            "type": "Cell",
            "value": serialize_cell_to_base64(value, "stack item `Cell`")
        }),
        StackItem::Continuation(value) => json!({
            "type": "Continuation",
            "value": serialize_cell_to_base64(&value.code().into_cell(), "stack item `Continuation`")
        }),
    })
}

pub fn deserialize_item(value: &Value) -> ClientResult<StackItem> {
    Ok(match value {
        Value::Null => StackItem::None,
        Value::Bool(v) => StackItem::Integer(Arc::new(if *v {
            IntegerData::one()
        } else {
            IntegerData::zero()
        })),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                StackItem::Integer(Arc::new(IntegerData::from_i64(i)))
            } else {
                return Err(Error::invalid_input_stack("Invalid number value", &value));
            }
        }
        Value::String(s) => StackItem::Integer(Arc::new(parse_integer_data(s)?)),
        Value::Array(array) => StackItem::Tuple(deserialize_items(array.iter())?),
        Value::Object(_) => return Err(Error::invalid_input_stack("Unexpected object", &value)),
    })
}

fn parse_integer_data(s: &String) -> ClientResult<IntegerData> {
    Ok(if s.eq("NaN") {
        IntegerData::nan()
    } else {
        let without_hex_prefix = s.replace("0x", "").replace("0X", "");
        IntegerData::from_str_radix(
            without_hex_prefix.as_str(),
            if s.len() == without_hex_prefix.len() {
                10
            } else {
                16
            },
        )
        .map_err(|err| Error::invalid_input_stack(err, &Value::String(s.clone())))?
    })
}
