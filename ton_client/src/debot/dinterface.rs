use super::base64_interface::Base64Interface;
use super::hex_interface::HexInterface;
use super::sdk_interface::SdkInterface;
use super::network_interface::NetworkInterface;
use crate::abi::{decode_message_body, Abi, ParamsOfDecodeMessageBody};
use crate::boc::{parse_message, ParamsOfParse};
use crate::debot::TonClient;
use crate::encoding::decode_abi_number;
use num_traits::cast::NumCast;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
pub type InterfaceResult = Result<(u32, Value), String>;

async fn decode_msg(
    client: TonClient,
    msg_body: String,
    abi: Abi,
) -> Result<(String, Value), String> {
    let decoded = decode_message_body(
        client.clone(),
        ParamsOfDecodeMessageBody {
            abi,
            body: msg_body,
            is_internal: true,
        },
    )
    .await
    .map_err(|e| format!("invalid message body: {}", e))?;
    let (func, args) = (decoded.name, decoded.value.unwrap_or(json!({})));
    debug!("{} ({})", func, args);
    Ok((func, args))
}

#[async_trait::async_trait]
pub trait DebotInterface {
    fn get_id(&self) -> String;
    fn get_abi(&self) -> Abi;
    async fn call(&self, func: &str, args: &Value) -> InterfaceResult;
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
            .await
            .map_err(|e| format!("{}", e))?;

        let body = parsed.parsed["body"]
            .as_str()
            .ok_or(format!("parsed message has no body"))?
            .to_owned();
        debug!("interface {} call", interface_id);
        match interfaces.get(interface_id) {
            Some(object) => {
                let abi = object.get_abi();
                let (func, args) = decode_msg(client.clone(), body, abi).await?;
                object.call(&func, &args)
                    .await
                    .map_err(|e| format!("interface {}.{} failed: {}", interface_id, func, e))
            }
            None => {
                debug!("interface {} not implemented", interface_id);
                Ok((0, json!({})))
            },
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

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(HexInterface::new());
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(NetworkInterface::new(client.clone()));
        interfaces.insert(iface.get_id(), iface);

        let iface: Arc<dyn DebotInterface + Send + Sync> =
            Arc::new(SdkInterface::new(client.clone()));
        interfaces.insert(iface.get_id(), iface);

        Self { client, interfaces }
    }

    pub fn add(&mut self, iface: Arc<dyn DebotInterface + Send + Sync>) {
        self.interfaces.insert(iface.get_id(), iface);
    }
}

pub fn decode_answer_id(args: &Value) -> Result<u32, String> {
    decode_abi_number::<u32>(
        args["answerId"]
            .as_str()
            .ok_or(format!("answer id not found in argument list"))?,
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

pub fn get_num_arg<T>(args: &Value, name: &str) -> Result<T, String>
where
    T: NumCast,
{
    let num_str = get_arg(args, name)?;
    decode_abi_number::<T>(&num_str)
        .map_err(|e| format!("failed to parse integer \"{}\": {}", num_str, e))
}

pub fn get_bool_arg(args: &Value, name: &str) -> Result<bool, String> {
    args[name]
        .as_bool()
        .ok_or(format!("\"{}\" not found", name))
}

pub fn get_array_strings(args: &Value, name: &str) -> Result<Vec<String>, String> {
    let array = args[name]
        .as_array()
        .ok_or(format!("\"{}\" is invalid: must be array", name))?;
    let mut strings = vec![];
    for elem in array {
        let string = hex::decode(
            elem.as_str().ok_or_else(|| format!("array element is invalid: must be string"))?
        ).map_err(|e| format!("{}", e))?;
        strings.push(
            std::str::from_utf8(&string).map_err(|e| format!("{}", e)).map(|x| x.to_string())?
        );
    }
    Ok(strings)
}