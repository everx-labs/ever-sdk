use super::dinterface::{InterfaceResult, DebotInterface, decode_answer_id, get_string_arg};
use serde_json::Value;
use crate::abi::{Abi};

const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "encode",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"str","type":"bytes"}
			],
			"outputs": [
				{"name":"base64","type":"bytes"}
			]
		},
		{
			"name": "decode",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"base64","type":"bytes"}
			],
			"outputs": [
				{"name":"str","type":"bytes"}
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

pub const BASE64_ID: &str = "8913b27b45267aad3ee08437e64029ac38fb59274f19adca0b23c4f957c8cfa1";

pub struct Base64Interface {}

impl Base64Interface {
    pub fn new() -> Self {
        Self { }
    }

    fn encode(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let str_to_encode = get_string_arg(args, "str")?;
        let encoded = base64::encode(&str_to_encode);
        Ok((answer_id, json!({ "base64": hex::encode(encoded.as_bytes()) })))
    }

    fn decode(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let str_to_decode = get_string_arg(args, "base64")?;
        let decoded = base64::decode(&str_to_decode).unwrap();
        Ok((answer_id, json!({ "str": hex::encode(&decoded) })))
    }
}

#[async_trait::async_trait]
impl DebotInterface for Base64Interface {
    fn get_id(&self) -> String {
        BASE64_ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        match func {
            "encode" => self.encode(args),
            "decode"  => self.decode(args),
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }

}
