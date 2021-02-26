use super::dinterface::{
    decode_answer_id, get_arg, get_string_arg, DebotInterface, InterfaceResult,
};
use crate::abi::Abi;
use serde_json::Value;

const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "encode",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"data","type":"bytes"}
			],
			"outputs": [
				{"name":"hexstr","type":"bytes"}
			]
		},
		{
			"name": "decode",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"hexstr","type":"bytes"}
			],
			"outputs": [
				{"name":"data","type":"bytes"}
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

const HEX_ID: &str = "edfbb00d6ebd16d57a1636774845af9499b400ba417da8552f40b1250256ff8f";

pub struct HexInterface {}

impl HexInterface {
    pub fn new() -> Self {
        Self {}
    }

    fn encode(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
		let encoded = get_arg(args, "data")?;
        Ok((
            answer_id,
            json!({ "hexstr": hex::encode(encoded.as_bytes()) }),
        ))
    }

    fn decode(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let str_to_decode = get_string_arg(args, "hexstr")?;
        let decoded =
            hex::decode(&str_to_decode).map_err(|e| format!("invalid hex: {}", e))?;
        Ok((answer_id, json!({ "data": hex::encode(&decoded) })))
    }
}

#[async_trait::async_trait]
impl DebotInterface for HexInterface {
    fn get_id(&self) -> String {
        HEX_ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        match func {
            "encode" => self.encode(args),
            "decode" => self.decode(args),
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}
