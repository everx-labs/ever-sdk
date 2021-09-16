use super::dinterface::{
    get_arg, get_num_arg, decode_answer_id, get_string_arg, DebotInterface, InterfaceResult,
};
use crate::abi::Abi;
use serde_json::{Value as JsonValue};
use ton_abi::{ParamType, Param, token::Tokenizer, TokenValue};
use crate::net::{ParamsOfQueryCollection, OrderBy, SortDirection, query_collection};
use super::TonClient;
use std::collections::HashMap;
use crate::boc::internal::{serialize_cell_to_base64};
use sha2::Digest;

const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "collection",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"collectionType","type":"uint8"},
				{"name":"queryFilter","type":"bytes"},
				{"name":"limit","type":"uint32"},
				{"name":"paginationId","type":"uint256"}
			],
			"outputs": [
				{"name":"status","type":"uint8"},
				{"components":[{"name":"kind","type":"uint8"},{"name":"value","type":"bytes"},{"name":"object","type":"map(uint256,cell)"},{"components":[{"name":"cell","type":"cell"}],"name":"array","type":"tuple[]"}],"name":"objects","type":"tuple[]"},
				{"name":"nextId","type":"uint256"}
			]
		}
	],
	"data": [
	],
	"events": [
	]
}
"#;

const ID: &str = "5c6fd81616cdfb963632109c42144a3a885c8d0f2e8deb5d8e15872fb92f2811";

use serde_repr::{Serialize_repr, Deserialize_repr};
#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum ValKind {
    String = 0,
    Number = 1,
    Bool = 2,
    Array = 3,
    Object = 4,
    Null = 5,
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
        println!("new_null");
        Self::default()
    }

    fn new_bool(v: bool) -> Option<Self> {
        println!("new_bool");
        let mut val = Self::default();
        val.kind = ValKind::Bool;
        val.value = Self::serialize(ParamType::Bool, json!(v))?;
        Some(val)
    }

    fn new_number(v: i64) -> Option<Self> {
        println!("new_number");
        let mut val = Self::default();
        val.kind = ValKind::Number;
        val.value = Self::serialize(ParamType::Int(256), json!(v))?;
        Some(val)
    }

    fn new_string(v: String) -> Option<Self> {
        println!("new_string");
        let mut val = Self::default();
        val.kind = ValKind::String;
        val.value = hex::encode(v);
        Some(val)
    }

    fn new_object(map: serde_json::map::Map<String, JsonValue>) -> Option<Self> {
        println!("new_object");
        let mut val = Self::default();
        val.kind = ValKind::Object;
        for (k, v) in map {
            let mut hasher = sha2::Sha256::new();
            hasher.update(k);
            let hash = hasher.finalize();
            let json: JsonValue = serde_json::to_value(pack(v)?).ok()?;
            let params = [
                Param::new("kind", ParamType::Uint(8)),
                Param::new("value", ParamType::Bytes),
                Param::new("object", ParamType::Map(
                    Box::new(ParamType::Uint(256)),
                    Box::new(ParamType::Cell),
                )),
                Param::new("array", ParamType::Array(Box::new(ParamType::Cell))),
            ];
            let tokens = Tokenizer::tokenize_all_params(&params, &json).unwrap();
            let builder = TokenValue::pack_values_into_chain(&tokens[..], vec![], 2).unwrap();
            let serialized = serialize_cell_to_base64(&ton_types::Cell::from(&builder), "QueryValue").ok()?;
            val.object.insert(format!("0x{}", hex::encode(&hash[..])), serialized);
        }
        Some(val)
    }

    fn serialize(param_type: ParamType, json: JsonValue) -> Option<String> {
        let tokens = Tokenizer::tokenize_all_params(
            &[Param::new("arg0", param_type)], 
            &json!({"arg0": json})
        ).ok()?;
        let builder = TokenValue::pack_values_into_chain(&tokens[..], vec![], 2).ok()?;
        Some(hex::encode(builder.data()))
        //serialize_cell_to_base64(&ton_types::Cell::from(&builder), "QueryValue").ok()
    }
}


fn pack(json_obj: JsonValue)-> Option<Value> {
    match json_obj {
        JsonValue::Null => Some(Value::new_null()),
        JsonValue::Bool(v) => Value::new_bool(v),
        JsonValue::String(v) => Value::new_string(v),
        JsonValue::Number(v) => Value::new_number(v.as_i64()?),
        JsonValue::Object(map) => Value::new_object(map),
        JsonValue::Array(_) => Some(Value::new_null()),
    }
}

#[derive(Serialize, Deserialize)]
#[repr(u8)]
enum QueryStatus {
    Success,
    InvalidFilter,
    InvalidLimit,
    InvalidSorting,
    NetworkError,
    UnknownError
}

pub struct QueryInterface {
    ton: TonClient,
}

impl QueryInterface {
    pub fn new(ton: TonClient) -> Self {
        Self { ton }
    }

    async fn collection(&self, args: &JsonValue) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let query_filter = get_string_arg(args, "queryFilter")?;
        let collection_type = get_num_arg::<u8>(args, "collectionType")?;
        let limit = get_num_arg::<u32>(args, "limit")?;
        let _pagination_id = get_arg(args, "paginationId")?;

        let collection_name = match collection_type {
            0 => "accounts",
            1 => "messages",
            2 => "transactions",
            _ => "unknown",
        }.to_owned();

        let result = self.query(collection_name, query_filter, limit).await;
        let (status, objects, id) = match result {
            Ok(json_objects) => {
                let nextid = json_objects.last().unwrap()["id"]
                    .as_str()
                    .map(|v| v.to_owned())
                    .unwrap_or_default();
                
                match Self::pack_objects(json_objects) {
                    Some(objects) => (QueryStatus::Success, objects, nextid),
                    None => (QueryStatus::UnknownError, vec![], format!("0")),
                }
            },
            Err(status) => {
                (status, vec![], format!("0"))
            },
        };

        Ok((
            answer_id,
            json!({
                "status": status as u8,
                "objects": serde_json::to_string(&objects).unwrap(),
                "nextId": id,
            }),
        ))
    }

    async fn query(&self, collection: String, filter: String, limit: u32) -> Result<Vec<JsonValue>, QueryStatus> {
        let filter: Option<JsonValue> = Some(
            serde_json::from_str(&filter).map_err(|_| QueryStatus::InvalidFilter)?
        );
        let result = query_collection(
            self.ton.clone(),
            ParamsOfQueryCollection{
                collection,
                filter,
                result: format!("id boc"),
                order: Some(vec![OrderBy { 
                    path: format!("id"), direction: SortDirection::ASC 
                }]),
                limit: Some(limit),
            },
        ).await.map_err(|_| QueryStatus::NetworkError)?;
        Ok(result.result)
    }

    fn pack_objects(json_objects: Vec<JsonValue>) -> Option<Vec<Value>> {
        let mut objects = vec![];
        for obj in json_objects {
            objects.push(pack(obj)?);
        }
        Some(objects)
    }
}

#[async_trait::async_trait]
impl DebotInterface for QueryInterface {
    fn get_id(&self) -> String {
        ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &JsonValue) -> InterfaceResult {
        match func {
            "collection" => self.collection(args).await,
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pack;

    #[test]
    fn test_pack() {
        let result_val = pack(json!({
          "a": true,
          "b": 1234567,
          "c": "Hello, world",
          "d": {
              "aa": "0x444",
              "bb": json!(null),
              "cc": {
                  "aaa": "Buy buy!"
              }
          }
        })).unwrap();
        println!("{}", serde_json::to_string(&result_val).unwrap());
    }
}