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

    fn new_number(v: i64) -> Option<Self> {
        let mut val = Self::default();
        val.kind = ValKind::Number;
        val.value = Self::serialize(ParamType::Int(256), json!(v))?;
        Some(val)
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
            let mut hasher = sha2::Sha256::new();
            hasher.update(k);
            let hash = hasher.finalize();
            let json: JsonValue = serde_json::to_value(pack(v)?).ok()?;
            println!("{}", json);
            let params = [
                Param::new("kind", ParamType::Uint(8)),
                Param::new("value", ParamType::Cell),
                Param::new(
                    "object",
                    ParamType::Map(Box::new(ParamType::Uint(256)), Box::new(ParamType::Cell)),
                ),
                Param::new("array", ParamType::Array(Box::new(ParamType::Tuple(vec![Param::new("cell", ParamType::Cell)])))),
            ];
            let tokens = Tokenizer::tokenize_all_params(&params, &json).unwrap();
            let builder =
                TokenValue::pack_values_into_chain(&tokens[..], vec![], &ABI_VERSION_2_0).unwrap();
            let serialized =
                serialize_cell_to_base64(&ton_types::Cell::from(&builder), "QueryValue").ok()?;
            val.object
                .insert(format!("0x{}", hex::encode(&hash[..])), serialized);
        }
        Some(val)
    }

    fn new_array(array: Vec<JsonValue>) -> Option<Self> {
        let mut val = Self::default();
        val.kind = ValKind::Array;
        for element in array {
            val.array.push(Cell {
                cell: pack(element)?.value,
            });
        }
        Some(val)
    }

    fn serialize(param_type: ParamType, json: JsonValue) -> Option<String> {
        let tokens = Tokenizer::tokenize_all_params(
            &[Param::new("arg0", param_type)],
            &json!({ "arg0": json }),
        )
        .ok()?;
        let builder =
            TokenValue::pack_values_into_chain(&tokens[..], vec![], &ABI_VERSION_2_0).ok()?;
        serialize_cell_to_base64(&ton_types::Cell::from(&builder), "QueryValue").ok()
    }
}

pub fn pack(json_obj: JsonValue) -> Option<Value> {
    match json_obj {
        JsonValue::Null => Some(Value::new_null()),
        JsonValue::Bool(v) => Value::new_bool(v),
        JsonValue::String(v) => Value::new_string(v),
        JsonValue::Number(v) => Value::new_number(v.as_i64()?),
        JsonValue::Object(map) => Value::new_object(map),
        JsonValue::Array(array) => Value::new_array(array),
    }
}
