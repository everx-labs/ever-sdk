use super::dinterface::{
    decode_answer_id, get_num_arg, get_string_arg, DebotInterface, InterfaceResult,
};
use super::TonClient;
use crate::abi::Abi;
use crate::debot::json_lib_utils::{pack, Value};
use crate::net::{query_collection, OrderBy, ParamsOfQueryCollection, SortDirection};
use serde_json::Value as JsonValue;

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
				{"name":"returnFilter","type":"bytes"},
				{"name":"limit","type":"uint32"},
				{"components":[{"name":"path","type":"bytes"},{"name":"direction","type":"uint8"}],"name":"orderBy","type":"tuple"}
			],
			"outputs": [
				{"name":"status","type":"uint8"},
				{"components":[{"name":"kind","type":"uint8"},{"name":"value","type":"cell"},{"name":"object","type":"map(uint256,cell)"},{"components":[{"name":"cell","type":"cell"}],"name":"array","type":"tuple[]"}],"name":"objects","type":"tuple[]"}
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

#[derive(Clone)]
#[repr(u8)]
enum QueryStatus {
    Success = 0,
    FilterError = 1,
    NetworkError = 2,
    PackingError = 3,
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
        let collection_type = get_num_arg::<u8>(args, "collectionType")?;
        let query_filter = get_string_arg(args, "queryFilter")?;
        let return_filter = get_string_arg(args, "returnFilter")?;
        let limit = get_num_arg::<u32>(args, "limit")?;
        let order_by = OrderBy {
            path: get_string_arg(&args["orderBy"], "path")?,
            direction: match get_num_arg::<u8>(&args["orderBy"], "direction")? {
                0 => SortDirection::ASC,
                _ => SortDirection::DESC,
            },
        };

        let collection_name = match collection_type {
            0 => "accounts",
            1 => "messages",
            2 => "transactions",
            _ => "unknown",
        }
        .to_owned();

        let result = self
            .query(
                collection_name,
                query_filter,
                return_filter,
                limit,
                order_by,
            )
            .await;
        let (status, objects) = match result {
            Ok(json_objects) => match Self::pack_objects(json_objects) {
                Some(objects) => (QueryStatus::Success, objects),
                None => (QueryStatus::PackingError, vec![]),
            },
            Err(status) => (status, vec![]),
        };

        Ok((
            answer_id,
            json!({
                "status": status as u8,
                "objects": objects,
            }),
        ))
    }

    async fn query(
        &self,
        collection: String,
        filter: String,
        result: String,
        limit: u32,
        order_by: OrderBy,
    ) -> Result<Vec<JsonValue>, QueryStatus> {
        let filter: Option<JsonValue> =
            Some(serde_json::from_str(&filter).map_err(|_| QueryStatus::FilterError)?);
        let result = query_collection(
            self.ton.clone(),
            ParamsOfQueryCollection {
                collection,
                filter,
                result,
                order: Some(vec![order_by]),
                limit: Some(limit),
            },
        )
        .await
        .map_err(|_| QueryStatus::NetworkError)?;
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
        }))
        .unwrap();
        println!("{}", serde_json::to_string(&result_val).unwrap());
    }
}
