use serde_json::Value;
use crate::debot::TonClient;
use std::collections::HashMap;
use crate::boc::{parse_message, ParamsOfParse};
use crate::abi::{Abi, ParamsOfDecodeMessageBody, decode_message_body};

const DEBOT_WC: i8 = -31;
pub type InterfaceResult = Result<(u32, String), String>;
pub type InterfaceMethod<'a> = Box<(dyn Fn(&'a Value) -> InterfaceResult + 'static)>;

pub(crate) fn boxed<'a, F>(f: F) -> InterfaceMethod<'a>
    where F: Fn(&'a Value) -> InterfaceResult + 'static {
        Box::new(f) as InterfaceMethod
}

pub trait DebotInterface {
    fn get_id(&self) -> String;
    fn get_abi(&self) -> Abi;
    fn call_function(&self, func: &str, args: &Value) -> InterfaceResult;

    fn call(&self, client: TonClient, msg_body: String) -> InterfaceResult {
        let decoded = decode_message_body(
            client.clone(),
            ParamsOfDecodeMessageBody {
                abi: self.get_abi(),
                body: msg_body,
                is_internal: true,
            },
        ).map_err(|e| format!(" failed to decode message body: {}", e))?;

        debug!("request: {} ({})", decoded.name, decoded.value.as_ref().unwrap());

        self.call_function(&decoded.name, decoded.value.as_ref().unwrap())
    }
}

pub struct BuiltinInterfaces {
    client: TonClient,
    interfaces: HashMap<String, Box<dyn DebotInterface>>
}

impl BuiltinInterfaces {
    pub fn new(client: TonClient) -> Self {
        let mut interfaces = HashMap::new();
//        interfaces.insert();
        Self {client, interfaces}
    }

    pub fn try_execute(&self, msg: &String) -> Option<InterfaceResult> {
        let res = self.execute(msg);
        match res.as_ref() {
            Err(_) => Some(res),
            Ok(val) => {
                if val.0 == 0 {
                    None
                } else {
                    Some(res)
                }
            },
        }
    }

    fn execute(&self, msg: &String) -> InterfaceResult {
        let parsed = parse_message(
            self.client.clone(),
            ParamsOfParse { boc: msg.clone() },
        ).map_err(|e| format!("{}", e))?;

        let body = parsed.parsed["body"]
            .as_str()
            .ok_or(format!("parsed message has no body"))?
            .to_owned();
        let iface_addr = parsed.parsed["dst"]
            .as_str()
            .ok_or(format!("parsed message has no dst address"))?;
        let wc_and_addr: Vec<_> = iface_addr.split(':').collect();
        let interface_id = wc_and_addr[1];
        let wc = i8::from_str_radix(wc_and_addr[0], 10)
            .map_err(|e| format!("interface dst address has invalid workchain id {}", e))?;
        
        if wc != DEBOT_WC {
            return Err(format!("invalid interface workchain id {}", wc));
        }

        debug!("call for interface id {}", interface_id);
        
        match self.interfaces.get(&interface_id) {
            Some(object) => object.call(self.client, body),
            None => Ok((0, json!({}))),
        }
    }
}

pub fn decode_answer_id(args: &Value) -> Result<u32, String> {
    u32::from_str_radix(
        args["answerId"].as_str()
            .ok_or(format!("answer id not found in argument list"))?, 
        10
    ).map_err(|e| format!("{}", e))
}