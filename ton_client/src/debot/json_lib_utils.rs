use crate::boc::internal::{deserialize_cell_from_base64, serialize_cell_to_base64};
use serde_json::Value as JsonValue;
use sha2::Digest;
use std::collections::HashMap;
use ton_abi::{contract::ABI_VERSION_2_0, token::Tokenizer, Param, ParamType, TokenValue};

use serde_repr::{Deserialize_repr, Serialize_repr};
#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ValKind {
    String = 0,
    Number = 1,
    Bool = 2,
    Array = 3,
    Object = 4,
    Null = 5,
    Cell = 6,
}

impl Default for ValKind {
    fn default() -> Self {
        ValKind::Null
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Cell {
    cell: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Value {
    kind: ValKind,
    value: String,
    object: HashMap<String, String>,
    array: Vec<Cell>,
}

impl Value {
    fn new_null() -> Self {
        Self::default()
    }

    fn new_bool(v: bool) -> Option<Self> {
        let mut val = Self::default();
        val.kind = ValKind::Bool;
        val.value = Self::serialize(ParamType::Bool, json!(v))?;
        Some(val)
    }

    fn new_number(v: serde_json::Number) -> Option<Self> {
        if v.is_i64() {
            let mut val = Self::default();
            val.kind = ValKind::Number;
            val.value = Self::serialize(ParamType::Int(256), json!(v.as_i64()?))?;
            Some(val)
        } else {
            Value::new_string(v.to_string())
        }
    }

    fn new_string(v: String) -> Option<Self> {
        let mut val = Self::default();
        val.kind = ValKind::String;
        if deserialize_cell_from_base64(&v, "QueryValue").is_ok() {
            val.value = v;
            val.kind = ValKind::Cell;
        } else {
            val.value = Self::serialize(ParamType::Bytes, json!(hex::encode(v)))?;
        }
        Some(val)
    }

    fn new_object(map: serde_json::map::Map<String, JsonValue>) -> Option<Self> {
        let mut val = Self::default();
        val.kind = ValKind::Object;
        for (k, v) in map {
            let json: JsonValue = serde_json::to_value(pack(v)?).ok()?;
            let packed = Self::pack_value_to_cell(json, Some(&k))?;
            let mut hasher = sha2::Sha256::new();
            hasher.update(k);
            let hash = hasher.finalize();
            val.object
                .insert(format!("0x{}", hex::encode(&hash[..])), packed);
        }
        Some(val)
    }

    fn new_array(array: Vec<JsonValue>) -> Option<Self> {
        let mut val = Self::default();
        val.kind = ValKind::Array;
        for element in array {
            let json: JsonValue = serde_json::to_value(pack(element)?).ok()?;
            let packed = Self::pack_value_to_cell(json, None)?;
            val.array.push(Cell { cell: packed });
        }
        Some(val)
    }

    fn pack_value_to_cell(mut json: JsonValue, key: Option<&String>) -> Option<String> {
        let mut params = vec![
            Param::new("kind", ParamType::Uint(8)),
            Param::new("value", ParamType::Cell),
            Param::new(
                "object",
                ParamType::Map(Box::new(ParamType::Uint(256)), Box::new(ParamType::Cell)),
            ),
            Param::new("array", ParamType::Array(Box::new(ParamType::Tuple(vec![Param::new("cell", ParamType::Cell)])))),
        ];
        if let Some(k) = key {
            params.push(Param::new("key", ParamType::Bytes));
            json["key"] = json!(hex::encode(k));
        }

        let tokens = Tokenizer::tokenize_all_params(&params, &json).unwrap();
        let builder =
            TokenValue::pack_values_into_chain(&tokens[..], vec![], &ABI_VERSION_2_0).unwrap();
        let serialized =
            serialize_cell_to_base64(&builder.into_cell().unwrap(), "QueryValue").ok()?;
        Some(serialized)
    }

    fn serialize(param_type: ParamType, json: JsonValue) -> Option<String> {
        let tokens = Tokenizer::tokenize_all_params(
            &[Param::new("arg0", param_type)],
            &json!({ "arg0": json }),
        )
        .ok()?;
        let builder =
            TokenValue::pack_values_into_chain(&tokens[..], vec![], &ABI_VERSION_2_0).ok()?;
        serialize_cell_to_base64(&builder.into_cell().unwrap(), "QueryValue").ok()
    }
}

pub fn pack(json_obj: JsonValue) -> Option<Value> {
    match json_obj {
        JsonValue::Null => Some(Value::new_null()),
        JsonValue::Bool(v) => Value::new_bool(v),
        JsonValue::String(v) => Value::new_string(v),
        JsonValue::Number(v) => Value::new_number(v),
        JsonValue::Object(map) => Value::new_object(map),
        JsonValue::Array(array) => Value::new_array(array),
    }
}

fn try_replace_hyphens(
    obj: &mut JsonValue,
    pointer: &str,
    name: &str,
) -> Result<(), String> {
    if name.contains('_') {
        match obj.pointer_mut(pointer) {
            Some(subobj) => {
                let map = subobj.as_object_mut().unwrap();
                if let Some(value) = map.remove(&name.replace('_', "-")) {
                    map.insert(name.to_owned(), value);
                }
            }
            None => Err(format!("key not found: \"{}\"", name))?,
        }
    }
    Ok(())
}

fn string_to_hex(obj: &mut JsonValue, pointer: &str) -> Result<(), String> {
    let val_str = obj
        .pointer(pointer)
        .ok_or_else(|| format!("argument not found"))?
        .as_str()
        .ok_or_else(|| format!("argument not a string"))?;
    *obj.pointer_mut(pointer).unwrap() = json!(hex::encode(val_str));
    Ok(())
}

pub(crate) fn bypass_json(
    top_pointer: &str,
    obj: &mut JsonValue,
    p: Param,
    string_or_bytes: ParamType,
) -> Result<(), String> {
    let pointer = format!("{}/{}", top_pointer, p.name);
    if let None = obj.pointer(&pointer) {
        try_replace_hyphens(obj, top_pointer, &p.name)?;
    }
    match p.kind {
        ParamType::Bytes | ParamType::String => if p.kind == string_or_bytes {
            string_to_hex(obj, &pointer).map_err(|e| format!("{}: \"{}\"", e, p.name))?;
        }
        ParamType::Tuple(params) => {
            for p in params {
                bypass_json(&pointer, obj, p, string_or_bytes.clone())?;
            }
        }
        ParamType::Array(ref elem_type) => {
            let elem_count = obj
                .pointer(&pointer)
                .ok_or_else(|| format!("\"{}\" not found", pointer))?
                .as_array()
                .ok_or_else(|| String::from("Failed to retrieve an array"))?
                .len();
            for i in 0..elem_count {
                bypass_json(
                    &pointer,
                    obj,
                    Param::new(&i.to_string(), (**elem_type).clone()),
                    string_or_bytes.clone(),
                )?;
            }
        }
        ParamType::Map(_, ref value) => {
            let keys: Vec<String> = obj
                .pointer(&pointer)
                .ok_or_else(|| format!("\"{}\" not found", pointer))?
                .as_object()
                .ok_or_else(|| String::from("Failed to retrieve an object"))?
                .keys()
                .map(|k| k.clone())
                .collect();
            for key in keys {
                bypass_json(
                    &pointer,
                    obj,
                    Param::new(key.as_str(), (**value).clone()),
                    string_or_bytes.clone(),
                )?;
            }
        }
        _ => (),
    }
    Ok(())
}