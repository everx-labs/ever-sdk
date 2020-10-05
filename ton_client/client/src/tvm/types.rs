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

use crate::error::ApiResult;
use crate::tvm::Error;
use core::result::Result::{Err, Ok};
use failure::_core::convert::TryFrom;
use serde_json::Value;
use std::slice::Iter;
use std::sync::Arc;
use ton_vm::stack::integer::IntegerData;
use ton_vm::stack::StackItem;

pub enum ExitCode {
    MessageExpired = 57,
    ReplayProtection = 52,
}

impl TryFrom<i32> for ExitCode {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            57 => Ok(Self::MessageExpired),
            62 => Ok(Self::ReplayProtection),
            _ => Err(value),
        }
    }
}

pub(crate) struct StackJson;

impl StackJson {
    pub(crate) fn json_array_from_items(items: Iter<StackItem>) -> ApiResult<Value> {
        let mut values = Vec::<Value>::new();
        for item in items {
            values.push(StackJson::json_value_from_item(item)?)
        }
        Ok(Value::Array(values))
    }

    pub(crate) fn items_from_json_array(values: Iter<Value>) -> ApiResult<Vec<StackItem>> {
        let mut items = Vec::<StackItem>::new();
        for value in values {
            items.push(Self::item_from_json_value(value)?)
        }
        Ok(items)
    }

    fn json_value_from_item(item: &StackItem) -> ApiResult<Value> {
        Ok(match item {
            StackItem::None => Value::Null,
            StackItem::Integer(i) => {
                let mut hex = i.to_str_radix(16);
                if hex.ne("NaN") {
                    hex.insert_str(if hex.starts_with("-") { 1 } else { 0 }, "0x")
                }
                Value::String(hex)
            }
            StackItem::Tuple(items) => Self::json_array_from_items(items.iter())?,
            StackItem::Builder(_) => json!({ "builder": Value::Null }),
            StackItem::Slice(_) => json!({ "slice": Value::Null }),
            StackItem::Cell(_) => json!({ "cell": Value::Null }),
            StackItem::Continuation(_) => json!({ "continuation": Value::Null }),
        })
    }

    fn parse_integer_data(s: &String) -> ApiResult<IntegerData> {
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
            .map_err(|err| Error::invalid_input_stack(err))?
        })
    }

    fn item_from_json_value(value: &Value) -> ApiResult<StackItem> {
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
                    return Err(Error::invalid_input_stack("Invalid number value"));
                }
            }
            Value::String(s) => StackItem::Integer(Arc::new(Self::parse_integer_data(s)?)),
            Value::Array(array) => StackItem::Tuple(Self::items_from_json_array(array.iter())?),
            Value::Object(_) => return Err(Error::invalid_input_stack("Unexpected object")),
        })
    }
}
