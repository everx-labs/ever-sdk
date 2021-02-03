use super::base64_interface::Base64Interface;
use super::sdk_interface::SdkInterface;
use crate::abi::{decode_message_body, Abi, ParamsOfDecodeMessageBody};
use crate::boc::{parse_message, ParamsOfParse};
use crate::debot::TonClient;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub type InterfaceResult = Result<(u32, Value), String>;

#[async_trait::async_trait]
pub trait DebotInterface {
    fn get_id(&self) -> String;
    fn get_abi(&self) -> Abi;
    async fn call(&self, func: &str, args: &Value) -> InterfaceResult;

    fn decode_msg(&self, client: TonClient, msg_body: String) -> Result<(String, Value), String> {
        let decoded = decode_message_body(
            client.clone(),
            ParamsOfDecodeMessageBody {
                abi: self.get_abi(),
                body: msg_body,
                is_internal: true,
            },
        )
        .map_err(|e| format!(" failed to decode message body: {}", e))?;
        let (func, args) = (decoded.name, decoded.value.unwrap_or(json!({})));
        debug!("{} ({})", func, args);
        Ok((func, args))
    }
}

#[async_trait::async_trait]
pub trait DebotInterfaceExecutor {
    fn get_interfaces<'a>(&'a self) -> &'a HashMap<String, Arc<dyn DebotInterface + Send + Sync>>;
    fn get_client(&self) -> TonClient;

    async fn try_execute(&self, msg: &String, interface_id: &String) -> Option<InterfaceResult> {
        let res = Self::execute(self.get_client(), msg, interface_id, self.get_interfaces()).await;
        match res.as_ref() {
            Err(_) => Some(res),
            Ok(val) => {
                if val.0 == 0 {
                    None
                } else {
                    Some(res)
                }
            }
        }
    }

    async fn execute(
        client: TonClient,
        msg: &String,
        interface_id: &String,
        interfaces: &HashMap<String, Arc<dyn DebotInterface + Send + Sync>>,
    ) -> InterfaceResult {
        let parsed = parse_message(client.clone(), ParamsOfParse { boc: msg.clone() })
            .map_err(|e| format!("{}", e))?;

        let body = parsed.parsed["body"]
            .as_str()
            .ok_or(format!("parsed message has no body"))?
            .to_owned();
        debug!("call for interface {}", interface_id);
        match interfaces.get(interface_id) {
            Some(object) => {
                let (func, args) = object.decode_msg(client.clone(), body)?;
                object.call(&func, &args).await
            }
            None => Ok((0, json!({}))),
        }
    }
}

pub struct BuiltinInterfaces {
    client: TonClient,
    interfaces: HashMap<String, Arc<dyn DebotInterface + Send + Sync>>,
}

#[async_trait::async_trait]
impl DebotInterfaceExecutor for BuiltinInterfaces {
    fn get_interfaces<'a>(&'a self) -> &'a HashMap<String, Arc<dyn DebotInterface + Send + Sync>> {
        &self.interfaces
    }
    fn get_client(&self) -> TonClient {
        self.client.clone()
    }
}

impl BuiltinInterfaces {
    pub fn new(client: TonClient) -> Self {
        let mut interfaces = HashMap::new();

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(Base64Interface::new());
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> =
            Arc::new(SdkInterface::new(client.clone()));
        interfaces.insert(iface.get_id(), iface);

        Self { client, interfaces }
    }
}

pub fn decode_answer_id(args: &Value) -> Result<u32, String> {
    u32::from_str_radix(
        args["answerId"]
            .as_str()
            .ok_or(format!("answer id not found in argument list"))?,
        10,
    )
    .map_err(|e| format!("{}", e))
}

pub fn get_arg(args: &Value, name: &str) -> Result<String, String> {
    args[name]
        .as_str()
        .ok_or(format!("\"{}\" not found", name))
        .map(|v| v.to_string())
}

pub fn get_string_arg(args: &Value, name: &str) -> Result<String, String> {
    let hex_str = args[name]
        .as_str()
        .ok_or(format!("\"{}\" not found", name))?;
    let bytes = hex::decode(hex_str).map_err(|e| format!("{}", e))?;
    std::str::from_utf8(&bytes)
        .map_err(|e| format!("{}", e))
        .map(|x| x.to_string())
}
