use super::dinterface::{decode_answer_id, get_string_arg, DebotInterface, InterfaceResult};
use crate::abi::Abi;
use crate::debot::json_lib_utils::pack;
use serde_json::Value as JsonValue;
use ton_abi::{Contract, Param, ParamType};

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
