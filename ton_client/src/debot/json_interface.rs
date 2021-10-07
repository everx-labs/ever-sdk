use super::dinterface::{decode_answer_id, get_string_arg, DebotInterface, InterfaceResult};
use crate::abi::Abi;
use crate::boc::internal::{deserialize_cell_from_base64, serialize_cell_to_base64};
use serde_json::Value as JsonValue;
use sha2::Digest;
use std::collections::HashMap;
use ton_abi::{
    contract::ABI_VERSION_2_0, token::Tokenizer, Contract, Param, ParamType, TokenValue,
};

const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "deserialize",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"json","type":"bytes"}
			],
			"outputs": [
				{"name":"result","type":"bool"}
			]
		},
		{
			"name": "parse",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"json","type":"bytes"}
			],
			"outputs": [
				{"name":"result","type":"bool"},
				{"components":[{"name":"kind","type":"uint8"},{"name":"value","type":"cell"},{"name":"object","type":"map(uint256,cell)"},{"components":[{"name":"cell","type":"cell"}],"name":"array","type":"tuple[]"}],"name":"obj","type":"tuple"}
			]
		}
	],
	"data": [
	],
	"events": [
	]
}
"#;

const BASE64_ID: &str = "442288826041d564ccedc579674f17c1b0a3452df799656a9167a41ab270ec19";

pub struct JsonInterface {
    debot_abi: String,
}

use serde_repr::{Deserialize_repr, Serialize_repr};
#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum ValKind {
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
struct Value {
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
            let params = [
                Param::new("kind", ParamType::Uint(8)),
                Param::new("value", ParamType::Cell),
                Param::new(
                    "object",
                    ParamType::Map(Box::new(ParamType::Uint(256)), Box::new(ParamType::Cell)),
                ),
                Param::new("array", ParamType::Array(Box::new(ParamType::Cell))),
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

fn pack(json_obj: JsonValue) -> Option<Value> {
    match json_obj {
        JsonValue::Null => Some(Value::new_null()),
        JsonValue::Bool(v) => Value::new_bool(v),
        JsonValue::String(v) => Value::new_string(v),
        JsonValue::Number(v) => Value::new_number(v.as_i64()?),
        JsonValue::Object(map) => Value::new_object(map),
        JsonValue::Array(array) => Value::new_array(array),
    }
}

impl JsonInterface {
    pub fn new(abi: &str) -> Self {
        Self {
            debot_abi: abi.to_owned(),
        }
    }

    fn deserialize(&self, args: &JsonValue) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let json_str = get_string_arg(args, "json")?;
        let mut json_obj: JsonValue = serde_json::from_str(&json_str)
            .map_err(|e| format!("argument \"json\" is not a valid json: {}", e))?;
        let _ = self.deserialize_json(&mut json_obj, answer_id)?;
        Ok((
            answer_id,
            json!({
                "result": true,
                "obj": json_obj,
            }),
        ))
    }

    fn parse(&self, args: &JsonValue) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let json_str = get_string_arg(args, "json")?;
        let json_obj: JsonValue = serde_json::from_str(&json_str)
            .map_err(|e| format!("argument \"json\" is not a valid json: {}", e))?;
        let result = pack(json_obj);
        Ok((
            answer_id,
            json!({
                "result": true,
                "obj": result,
            }),
        ))
    }

    fn deserialize_json(&self, json_obj: &mut JsonValue, answer_id: u32) -> Result<(), String> {
        let contract = Contract::load(self.debot_abi.as_bytes()).map_err(|e| format!("{}", e))?;
        let func = contract
            .function_by_id(answer_id, true)
            .map_err(|_| format!("function with id {} not found", answer_id))?;
        let obj = func
            .inputs
            .iter()
            .find(|e| e.name == "obj")
            .ok_or(format!("\"obj\" argument not found"))?;
        if let ParamType::Tuple(params) = &obj.kind {
            for p in params {
                let pointer = "";
                self.bypass_json(pointer, json_obj, p.clone())?;
            }
        }
        Ok(())
    }

    fn bypass_json(&self, top_pointer: &str, obj: &mut JsonValue, p: Param) -> Result<(), String> {
        let pointer = format!("{}/{}", top_pointer, p.name);
        if let None = obj.pointer(&pointer) {
            self.try_replace_hyphens(obj, top_pointer, &p.name)?;
        }
        match p.kind {
            ParamType::Bytes => {
                Self::string_to_hex(obj, &pointer).map_err(|e| format!("{}: \"{}\"", e, p.name))?;
            }
            ParamType::Tuple(params) => {
                for p in params {
                    self.bypass_json(&pointer, obj, p)?;
                }
            }
            ParamType::Array(ref elem_type) => {
                let elem_count = obj
                    .pointer(&pointer)
                    .ok_or_else(|| format!("\"{}\" not found", pointer))?
                    .as_array()
                    .unwrap()
                    .len();
                for i in 0..elem_count {
                    self.bypass_json(
                        &pointer,
                        obj,
                        Param::new(&i.to_string(), (**elem_type).clone()),
                    )?;
                }
            }
            ParamType::Map(_, ref value) => {
                let keys: Vec<String> = obj
                    .pointer(&pointer)
                    .ok_or_else(|| format!("\"{}\" not found", pointer))?
                    .as_object()
                    .unwrap()
                    .keys()
                    .map(|k| k.clone())
                    .collect();
                for key in keys {
                    self.bypass_json(&pointer, obj, Param::new(key.as_str(), (**value).clone()))?;
                }
            }
            _ => (),
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

    fn try_replace_hyphens(
        &self,
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
}

#[async_trait::async_trait]
impl DebotInterface for JsonInterface {
    fn get_id(&self) -> String {
        BASE64_ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &JsonValue) -> InterfaceResult {
        match func {
            "deserialize" => self.deserialize(args),
            "parse" => self.parse(args),
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::JsonInterface;

    const ABI: &str = r#"
    {
        "ABI version": 2,
        "header": ["time"],
        "functions": [
            {
                "name": "setResult",
                "id": "0x11111111",
                "inputs": [
                    {"components":[{"components":[{"name":"Provider","type":"bytes"},{"name":"Name","type":"bytes"},{"name":"Number","type":"uint64"},{"name":"Special_Name","type":"bytes"},{"name":"Url","type":"bytes"},{"components":[{"name":"Currency","type":"bytes"},{"name":"MinValueStr","type":"bytes"},{"name":"MaxValueStr","type":"bytes"}],"name":"Product","type":"tuple[]"}],"name":"Result","type":"tuple[]"},{"name":"Status","type":"bytes"},{"name":"TestValue2","type":"bytes"},{"name":"Numbers","type":"int32[]"}],"name":"obj","type":"tuple"}
                ],
                "outputs": [
                ]
            },
            {
                "name": "constructor",
                "inputs": [
                ],
                "outputs": [
                ]
            }
        ],
        "data": [
        ],
        "events": [
        ]
    }    
    "#;

    #[test]
    fn test_debot_json_desert_1() {
        let json_iface = JsonInterface::new(ABI);
        let mut json = json!({
            "Result": [{
                "Provider":"PROVIDER",
                "Name":"This is a name",
                "Number": 123,
                "Special-Name": "Name with hyphen",
                "Url":"https://this.is.url/logo/l.png",
                "Product":[{
                    "Currency":"TON",
                    "MinValue":2.00,
                    "MinValueStr":"2.00",
                    "MaxValue":461.00,
                    "MaxValueStr":"461.00",
                }]
            }],
            "Status":"success",
            "TestValue1": 9.200000000,
            "TestValue2": "9.300000000",
            "Numbers": [1, 2, 3],
            "Floats": [1.1, 2.1, 3.1]
        });
        json_iface.deserialize_json(&mut json, 0x11111111).unwrap();
        assert_eq!(
            json,
            json!({
                "Result": [{
                    "Provider":hex::encode("PROVIDER"),
                    "Name":hex::encode("This is a name"),
                    "Number": 123,
                    "Special_Name": hex::encode("Name with hyphen"),
                    "Url":hex::encode("https://this.is.url/logo/l.png"),
                    "Product":[{
                        "Currency":hex::encode("TON"),
                        "MinValue":2.00,
                        "MinValueStr":hex::encode("2.00"),
                        "MaxValue":461.00,
                        "MaxValueStr":hex::encode("461.00"),
                    }]
                }],
                "Status":hex::encode("success"),
                "TestValue1": 9.200000000,
                "TestValue2":hex::encode("9.300000000"),
                "Numbers": [1, 2, 3],
                "Floats": [1.1, 2.1, 3.1]
            })
        );
    }
}
