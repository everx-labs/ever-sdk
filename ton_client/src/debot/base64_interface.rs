use super::dinterface::{InterfaceResult, DebotInterface, InterfaceMethod, boxed, decode_answer_id};

const ABI: &str = r#"

"#;

pub const BASE64_ID: &str = "";

pub struct Base64Interface {
    methods: HashMap<String, InterfaceMethod>,
}

impl Base64Interface {
    pub fn new() -> Self {
        let mut methods = HashMap::new();
        h.insert("encode", boxed(Self::encode);
        h.insert("decode", boxed(Self::decode);
        Self {methods}
    }

    fn encode(args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let bytes = hex::decode(args["str"].as_str().unwrap()).unwrap();
        let str_to_encode = std::str::from_utf8(&bytes)
            .map_err(|e| format!("{}", e))?;
        let encoded = base64::encode(&str_to_encode);
        Ok((answer_id, json!({ "base64": hex::encode(encoded.as_bytes()) })))
    }

    fn decode(args: &Value) -> InterfaceResult {

    }
}

impl DebotInterface for Base64Interface {
    fn get_id(&self) -> String {
        BASE64_ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    fn call_function(&self, func: &str, args: &Value) -> Result<(u32, Value), String> {
        match self.methods.get(func) {
            Some(fun) => fun(args),
            None => Err(format!("function \"{}\" is not implemented", func)),
        }
    }

}
