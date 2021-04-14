use super::dinterface::{
    decode_answer_id, get_array_strings, get_string_arg, DebotInterface, InterfaceResult,
};
use crate::abi::Abi;
use serde_json::Value;
use std::str::FromStr;

const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "get",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"url","type":"bytes"},
				{"name":"headers","type":"bytes[]"}
			],
			"outputs": [
				{"name":"statusCode","type":"int32"},
				{"name":"retHeaders","type":"bytes[]"},
				{"name":"content","type":"bytes"}
			]
		},
		{
			"name": "post",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"url","type":"bytes"},
				{"name":"headers","type":"bytes[]"},
				{"name":"body","type":"bytes"}
			],
			"outputs": [
				{"name":"statusCode","type":"int32"},
				{"name":"retHeaders","type":"bytes[]"},
				{"name":"content","type":"bytes"}
			]
		}
	],
	"data": [
	],
	"events": [
	]
}
"#;

const ID: &str = "e38aed5884dc3e4426a87c083faaf4fa08109189fbc0c79281112f52e062d8ee";

pub struct NetworkInterface {}

impl NetworkInterface {
    pub fn new() -> Self {
        Self {}
    }

    async fn post(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let url = get_string_arg(args, "url")?;
        let headers = get_array_strings(args, "headers")?;
        let body = get_string_arg(args, "body")?;
        let answer = self.send(url, headers, Some(body)).await?;
        Ok((answer_id, answer))
    }

    async fn get(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let url = get_string_arg(args, "url")?;
        let headers = get_array_strings(args, "headers")?;
        let answer = self.send(url, headers, None).await?;
        Ok((answer_id, answer))
    }

    async fn send(&self, url: String, headers: Vec<String>, body: Option<String>) -> Result<Value, String> {
        let http_client = reqwest::Client::new();
        let mut map = reqwest::header::HeaderMap::new();
        for h in headers {
            let mut iter = h.split(':');
            let key = iter.next();
            let value = iter.next();
            if key.is_some() && value.is_some() {
                map.insert(
                    reqwest::header::HeaderName::from_str(key.unwrap().trim()).unwrap(),
                    value.unwrap().trim().parse().unwrap()
                );
            }
        }
        let response = match body {
            Some(body) => {
                http_client.post(&url)
                    .headers(map)
                    .body(body)
                    .send()
                    .await
                    .map_err(|e| format!("{}", e))?
            },
            None => {
                http_client.get(&url)
                    .headers(map)
                    .send()
                    .await
                    .map_err(|e| format!("{}", e))?
            },
        };
        let mut ret_headers: Vec<String> = vec![];
        for (k, v) in response.headers().iter() {
            ret_headers.push(hex::encode(format!("{}: {:?}", k, v)));
        }
        let status = response.status().as_str().to_owned();
        let content = response.bytes().await.unwrap().to_vec();
        Ok(json!({
            "statusCode": status,
            "retHeaders": ret_headers,
            "content": hex::encode(content),
        }))
    }
}

#[async_trait::async_trait]
impl DebotInterface for NetworkInterface {
    fn get_id(&self) -> String {
        ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        match func {
            "get" => self.get(args).await,
            "post" => self.post(args).await,
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}
