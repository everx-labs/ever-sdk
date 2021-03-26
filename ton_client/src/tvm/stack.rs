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

use crate::boc::internal::{deserialize_cell_from_base64, serialize_cell_to_base64};
use crate::error::ClientResult;
use crate::tvm::Error;
use core::result::Result::{Err, Ok};
use serde_json::Value;
use ton_types::BuilderData;
use std::ops::Deref;
use std::slice::Iter;
use ton_vm::stack::{continuation::ContinuationData, integer::IntegerData};
use ton_vm::stack::StackItem;

enum ProcessingResult<'a> {
    Serialized(Value),
    Nested(Box<dyn Iterator<Item = &'a StackItem> + 'a>),
    //LevelUp,
}

#[derive(Serialize, Deserialize)]
#[serde(tag="type", content = "value")]
enum ComplexType {
    List(Vec<Value>),
    Cell(String),
    Builder(String),
    Slice(String),
    Continuation(String),
}

fn is_equal_type(left: &Value, right: &Value) -> bool {
    left["type"] == right["type"] &&
    left.is_array() == right.is_array() &&
    left.is_string() == right.is_string()
}

pub fn serialize_items<'a>(
    items: Box<dyn Iterator<Item = &'a StackItem> + 'a>,
    flatten_lists: bool,
) -> ClientResult<Value> {
    let mut stack = vec![(vec![], items)];
    let mut list_items: Option<Vec<Value>> = None;
    loop {
        let (mut vec, mut iter) = stack.pop().unwrap();
        let next = iter.next();
        if let Some(list) = list_items.take() {
            // list is ended if current tuple has next element
            // or it already contains more than one element
            // or element type in current tuple is not equal to list items type
            if next.is_some() || vec.len() != 1 || !is_equal_type(&vec[0], &list[0]) {
                vec.push(json!(ComplexType::List(list)));
            } else {
                list_items = Some(list);
            }
        }
        
        if let Some(item) = next {
            match process_item(item)? {
                ProcessingResult::Serialized(value) => {
                    vec.push(value);
                    stack.push((vec, iter));
                },
                ProcessingResult::Nested(nested_iter) => {
                    stack.push((vec, iter));
                    stack.push((vec![], nested_iter));
                }
            }
        } else {
            if let Some((parent_vec, _)) = stack.last_mut() {
                // list starts from tuple with 2 elements: some value and null,
                // the value becomes the last list item
                if vec.len() == 2 && vec[1] == Value::Null && flatten_lists {
                    vec.resize(1, Value::Null);
                    list_items = Some(vec);
                } else if let Some(list) = list_items.take() {
                    vec.extend(list.into_iter());
                    list_items = Some(vec);
                } else {
                    parent_vec.push(Value::Array(vec));
                }
            } else {
                return Ok(Value::Array(vec));
            }
        }
    }
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
    // all negative numbers and positive numbers less than u128::MAX are encoded as decimal
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

pub fn serialize_item<'a>(item: &'a StackItem) -> ClientResult<Value> {
    Ok(serialize_items(Box::new(vec![item].into_iter()), false)?[0].take())
}

fn process_item(item: &StackItem) -> ClientResult<ProcessingResult> {
    Ok(match item {
        StackItem::None => ProcessingResult::Serialized(Value::Null),
        StackItem::Integer(value) => 
            ProcessingResult::Serialized(Value::String(serialize_integer_data(value))),
        StackItem::Tuple(items) => ProcessingResult::Nested(Box::new(items.iter())),
        StackItem::Builder(value) => ProcessingResult::Serialized(json!(
            ComplexType::Builder(
                serialize_cell_to_base64(&value.deref().into(), "stack item `Builder`")?
            )
        )),
        StackItem::Slice(value) => ProcessingResult::Serialized(json!(
            ComplexType::Slice(
                serialize_cell_to_base64(&value.into_cell(), "stack item `Slice`")?
            )
        )),
        StackItem::Cell(value) => ProcessingResult::Serialized(json!(
            ComplexType::Cell(
                serialize_cell_to_base64(value, "stack item `Cell`")?
            )
        )),
        StackItem::Continuation(value) => ProcessingResult::Serialized(json!(
            ComplexType::Continuation(
                serialize_cell_to_base64(&value.code().into_cell(), "stack item `Continuation`")?
            )
        )),
    })
}

pub fn deserialize_item(value: &Value) -> ClientResult<StackItem> {
    Ok(match value {
        Value::Null => StackItem::None,
        Value::Bool(v) => StackItem::boolean(*v),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                StackItem::int(i)
            } else {
                return Err(Error::invalid_input_stack("Invalid number value", value));
            }
        }
        Value::String(s) => StackItem::integer(parse_integer_data(&s)?),
        Value::Array(array) => StackItem::tuple(deserialize_items(array.iter())?),
        Value::Object(_) => {
            let object = serde_json::from_value(value.clone())
                .map_err(|err| Error::invalid_input_stack(
                    format!("Can not parse object: {}", err),value))?;
            match object {
                ComplexType::Builder(string) => {
                    let cell = deserialize_cell_from_base64(&string, "Builder")?.1;
                    StackItem::builder(BuilderData::from(&cell))
                }
                ComplexType::Cell(string) => {
                    let cell = deserialize_cell_from_base64(&string, "Cell")?.1;
                    StackItem::cell(cell)
                }
                ComplexType::Continuation(string) => {
                    let cell = deserialize_cell_from_base64(&string, "Continuation")?.1;
                    StackItem::continuation(ContinuationData::with_code(cell.into()))
                }
                ComplexType::Slice(string) => {
                    let cell = deserialize_cell_from_base64(&string, "Slice")?.1;
                    StackItem::slice(cell.into())
                }
                ComplexType::List(mut vec) => {
                    let mut list = StackItem::None;
                    while let Some(item) = vec.pop() {
                        list = StackItem::tuple(vec![deserialize_item(&item)?, list]);
                    }
                    list
                }
            }
        }
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
