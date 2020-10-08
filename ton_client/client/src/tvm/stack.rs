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

use crate::error::ClientResult;
use crate::tvm::Error;
use core::result::Result::{Err, Ok};
use serde_json::Value;
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

pub fn serialize_item(item: &StackItem) -> ClientResult<Value> {
    Ok(match item {
        StackItem::None => Value::Null,
        StackItem::Integer(i) => {
            let mut hex = i.to_str_radix(16);
            if hex.ne("NaN") {
                hex.insert_str(if hex.starts_with("-") { 1 } else { 0 }, "0x")
            }
            Value::String(hex)
        }
        StackItem::Tuple(items) => serialize_items(items.iter())?,
        StackItem::Builder(_) => json!({ "builder": Value::Null }),
        StackItem::Slice(_) => json!({ "slice": Value::Null }),
        StackItem::Cell(_) => json!({ "cell": Value::Null }),
        StackItem::Continuation(_) => json!({ "continuation": Value::Null }),
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
