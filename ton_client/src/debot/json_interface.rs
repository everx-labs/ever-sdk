use super::dinterface::{
    decode_answer_id, get_string_arg, DebotInterface, InterfaceResult,
};
use crate::abi::Abi;
use serde_json::Value;
use ton_abi::{Contract, param_type::ParamType, Param};

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

impl JsonInterface {
    pub fn new(abi: &str) -> Self {
        Self { debot_abi: abi.to_owned() }
    }

    fn deserialize(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let json_str = get_string_arg(args, "json")?;
        let mut json_obj: Value = serde_json::from_str(&json_str)
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

    fn deserialize_json(&self, json_obj: &mut Value, answer_id: u32) -> Result<(), String> {
        let contract = Contract::load(self.debot_abi.as_bytes())
            .map_err(|e| format!("{}", e))?;
        let func = contract.function_by_id(answer_id, true)
            .map_err(|_| format!("function with id {} not found", answer_id))?;
        let obj = func.inputs.iter().find(|e| e.name == "obj").ok_or(format!("\"obj\" argument not found"))?;
        if let ParamType::Tuple(params) = &obj.kind {
            for p in params {
                let pointer = "";
                self.bypass_json(pointer, json_obj, p.clone())?;
            }
        }
        Ok(())
    }

    fn bypass_json(&self, pointer: &str, obj: &mut Value, p: Param) -> Result<(), String> {
        let pointer = format!("{}/{}", pointer, p.name);
        match p.kind {
            ParamType::Bytes => {
                Self::string_to_hex(obj, &pointer)
                .map_err(|e| format!("\"{}\": {}", e, p.name))?;
            },
            ParamType::Tuple(params) => {
                for p in params {
                    self.bypass_json(&pointer, obj, p)?;
                }
            },
            ParamType::Array(ref elem_type) => {
                if let ParamType::Bytes = **elem_type {
                    let strings_count = obj.pointer(&pointer)
                        .ok_or_else(|| format!("\"{}\" not found", pointer))?
                        .as_array()
                        .unwrap().len();
                    for i in 0..strings_count {
                        self.bypass_json(&pointer, obj, Param::new(&i.to_string(), ParamType::Bytes))?;
                    }
                }
            },
            ParamType::Map(_, ref value) => {
                let keys: Vec<String> = obj.pointer(&pointer)
                    .ok_or_else(|| format!("\"{}\" not found", pointer))?
                    .as_object()
                    .unwrap()
                    .keys()
                    .map(|k| k.clone())
                    .collect();
                for key in keys {
                    self.bypass_json(&pointer, obj, Param::new(key.as_str(), (**value).clone()))?;
                }
            },
            _ => (),
        }
        Ok(())
    }

    fn string_to_hex(obj: &mut Value, pointer: &str) -> Result<(), String> {
        let val_str = obj.pointer(pointer)
            .ok_or_else(|| format!("argument not found"))?
            .as_str()
            .ok_or_else(|| format!("argument not a string"))?;
        *obj.pointer_mut(pointer).unwrap() = json!(hex::encode(val_str));
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

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        match func {
            "deserialize" => self.deserialize(args),
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}
