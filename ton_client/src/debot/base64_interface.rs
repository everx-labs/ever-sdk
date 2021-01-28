use super::dinterface::{InterfaceResult, DebotInterface, InterfaceMethod, boxed, decode_answer_id, get_string_arg};
use std::collections::HashMap;
use serde_json::Value;
use crate::abi::{Abi};

const ABI: &str = r#"

"#;

pub const BASE64_ID: &str = "";

pub struct Base64Interface {
    methods: HashMap<String, InterfaceMethod>,
}

impl Base64Interface {
    pub fn new() -> Self {
        let mut methods = HashMap::new();
        methods.insert("encode".to_owned(), boxed(Self::encode));
        methods.insert("decode".to_owned(), boxed(Self::decode));
        Self { methods: methods }
    }

    fn encode(args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let str_to_encode = get_string_arg(args, "str")?;
        let encoded = base64::encode(&str_to_encode);
        Ok((answer_id, json!({ "base64": hex::encode(encoded.as_bytes()) })))
    }

    fn decode(args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let str_to_decode = get_string_arg(args, "str")?;
        let decoded = base64::decode(&str_to_decode).unwrap();
        Ok((answer_id, json!({ "str": hex::encode(&decoded) })))
    }
}

impl DebotInterface for Base64Interface {
    fn get_id(&self) -> String {
        BASE64_ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    fn call_function(&self, func: &str, args: &Value) -> InterfaceResult {
        match self.methods.get(func) {
            Some(fun) => fun(args),
            None => Err(format!("function \"{}\" is not implemented", func)),
        }
    }

}
