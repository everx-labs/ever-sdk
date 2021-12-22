use super::dinterface::{
    decode_answer_id, get_num_arg, get_arg, DebotInterface, InterfaceResult,
};
use super::TonClient;
use crate::abi::Abi;
use crate::debot::json_lib_utils::{pack, Value};
use crate::net::{wait_for_collection, query_collection, query, OrderBy, ParamsOfQueryCollection, ParamsOfWaitForCollection, SortDirection, ParamsOfQuery};
use serde_json::Value as JsonValue;

const ABI: &str = r#"
{
	"ABI version": 2,
	"version": "2.2",
	"header": ["time"],
	"functions": [
		{
			"name": "collection",
			"id": "0x03186d00",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"collectionType","type":"uint8"},
				{"name":"queryFilter","type":"string"},
				{"name":"returnFilter","type":"string"},
				{"name":"limit","type":"uint32"},
				{"components":[{"name":"path","type":"string"},{"name":"direction","type":"uint8"}],"name":"orderBy","type":"tuple"}
			],
			"outputs": [
				{"name":"status","type":"uint8"},
				{"components":[{"name":"kind","type":"uint8"},{"name":"value","type":"cell"},{"name":"object","type":"map(uint256,cell)"},{"components":[{"name":"cell","type":"cell"}],"name":"array","type":"tuple[]"}],"name":"objects","type":"tuple[]"}
			]
		},
		{
			"name": "waitForCollection",
			"id": "0x4d635ba4",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"collectionType","type":"uint8"},
				{"name":"queryFilter","type":"string"},
				{"name":"returnFilter","type":"string"},
				{"name":"timeout","type":"uint32"}
			],
			"outputs": [
				{"name":"status","type":"uint8"},
				{"components":[{"name":"kind","type":"uint8"},{"name":"value","type":"cell"},{"name":"object","type":"map(uint256,cell)"},{"components":[{"name":"cell","type":"cell"}],"name":"array","type":"tuple[]"}],"name":"object","type":"tuple"}
			]
		},
        {
			"name": "query",
            "id": "0x784c89f6",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"query","type":"string"},
				{"name":"variables","type":"string"}
			],
			"outputs": [
				{"name":"status","type":"uint8"},
				{"components":[{"name":"kind","type":"uint8"},{"name":"value","type":"cell"},{"name":"object","type":"map(uint256,cell)"},{"components":[{"name":"cell","type":"cell"}],"name":"array","type":"tuple[]"}],"name":"object","type":"tuple"}
			]
		}
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
    VariablesError = 4,
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
        let query_filter = get_arg(args, "queryFilter")?;
        let return_filter = get_arg(args, "returnFilter")?;
        let limit = get_num_arg::<u32>(args, "limit")?;
        let order_by = OrderBy {
            path: get_arg(&args["orderBy"], "path")?,
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
            .collection_query(
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

    async fn collection_query(
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

    async fn run_wait_for_collection(
        &self,
        collection: String,
        filter: String,
        result: String,
        timeout: u32,
    ) -> Result<JsonValue, QueryStatus> {
        let filter: Option<JsonValue> =
            Some(serde_json::from_str(&filter).map_err(|_| QueryStatus::FilterError)?);
        let result = wait_for_collection(
            self.ton.clone(),
            ParamsOfWaitForCollection {
                collection,
                filter,
                result,
                timeout: Some(timeout),
            },
        )
        .await
        .map_err(|_| QueryStatus::NetworkError)?;
        Ok(result.result)
    }


    async fn wait_for_collection(&self, args: &JsonValue) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let collection_type = get_num_arg::<u8>(args, "collectionType")?;
        let query_filter = get_arg(args, "queryFilter")?;
        let return_filter = get_arg(args, "returnFilter")?;
        let timeout = get_num_arg::<u32>(args, "timeout")?;

        let collection_name = match collection_type {
            0 => "accounts",
            1 => "messages",
            2 => "transactions",
            _ => "unknown",
        }
        .to_owned();

        let result = self
            .run_wait_for_collection(
                collection_name,
                query_filter,
                return_filter,
                timeout,
            )
            .await;

        let (status, object) = match result {
            Ok(json_object) => match pack(json_object) {
                Some(object) => (QueryStatus::Success, object),
                None => (QueryStatus::PackingError, pack(json!({})).unwrap()),
            },
            Err(status) => (status, pack(json!({})).unwrap()),
        };

        Ok((
            answer_id,
            json!({
                "status": status as u8,
                "object": object,
            }),
        ))
    }


    fn get_query_variables(&self, variables: String) -> Result<Option<JsonValue>, QueryStatus> {
        if !variables.is_empty() {
            serde_json::from_str(&variables).map(|x| Some(x)).map_err(|_| QueryStatus::VariablesError)
        } else {
            Ok(None)
        }
    }

    async fn run_query(
        &self,
        query_str: String,
        variables: String,
    ) -> Result<JsonValue, QueryStatus> {
        let variables = self.get_query_variables(variables)?;

        let result = query(
            self.ton.clone(),
            ParamsOfQuery {
                query: query_str,
                variables: variables,
            },
        )
        .await
        .map_err(|_| QueryStatus::NetworkError)?;

        Ok(result.result)
    }

    async fn query(&self, args: &JsonValue) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let query_str = get_arg(args, "query")?;
        let variables_str = get_arg(args, "variables")?;

        let result = self
            .run_query(
                query_str,
                variables_str
            )
            .await;

        let (status, object) = match result {
            Ok(json_object) => match pack(json_object) {
                Some(object) => (QueryStatus::Success, object),
                None => (QueryStatus::PackingError, pack(json!({})).unwrap()),
            },
            Err(status) => (status, pack(json!({})).unwrap()),
        };

        Ok((
            answer_id,
            json!({
                "status": status as u8,
                "object": object,
            }),
        ))
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
            "waitForCollection" => self.wait_for_collection(args).await,
            "query" => self.query(args).await,
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
