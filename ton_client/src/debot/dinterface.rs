use super::base64_interface::Base64Interface;
use super::hex_interface::HexInterface;
use super::sdk_interface::SdkInterface;
use super::network_interface::NetworkInterface;
use super::query_interface::QueryInterface;
use super::json_lib_utils::bypass_json;
use super::JsonValue;
use crate::{abi::{Abi, Error}, error::ClientResult};
use crate::boc::{parse_message, ParamsOfParse};
use crate::debot::TonClient;
use crate::encoding::decode_abi_number;
use num_traits::cast::NumCast;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
pub type InterfaceResult = Result<(u32, Value), String>;
use ton_abi::{Contract, ParamType};
use ton_types::SliceData;

use crate::boc::internal::deserialize_cell_from_boc;
use ton_sdk::AbiContract;
use ton_abi::token::Detokenizer;

async fn decode_msg(
    client: TonClient,
    msg_body: String,
    abi: Abi,
) -> ClientResult<(String, Value)> {
    let abi = abi.json_string()?;
    let abi = AbiContract::load(abi.as_bytes()).map_err(|e| Error::invalid_json(e))?;
    let (_, body) = deserialize_cell_from_boc(&client, &msg_body, "message body").await?;
    let body: SliceData = body.into();
    let input = abi.decode_input(body, true, false)
        .map_err(|e| Error::invalid_message_for_decode(e))?;
    let value = Detokenizer::detokenize_to_json_value(&input.tokens)
        .map_err(|e| Error::invalid_message_for_decode(e))?;
    Ok((input.function_name, value))
}

#[async_trait::async_trait]
pub trait DebotInterface {
    fn get_id(&self) -> String;
    fn get_abi(&self) -> Abi;
    fn get_target_abi(&self, abi_version: &str) -> Abi {
        let mut abi = self.get_abi();
        if abi_version == "2.0" {
            return abi;
        }

        if let Abi::Json(ref json) = abi {
            let mut val: JsonValue = serde_json::from_str(json).unwrap_or(json!({}));
            if let Some(functions) = val.get_mut("functions") {
                if let Some(functions) = functions.as_array_mut() {
                    for func in functions {
                        if let Some(mut_func) = func.as_object_mut() {
                            mut_func.remove("id");
                        }
                    }
                    if let Ok(v) = serde_json::to_string(&val) {
                        abi = Abi::Json(v);
                    }
                }
            }
        }
        abi
    }
    async fn call(&self, func: &str, args: &Value) -> InterfaceResult;
}

#[async_trait::async_trait]
pub trait DebotInterfaceExecutor {
    fn get_interfaces<'a>(&'a self) -> &'a HashMap<String, Arc<dyn DebotInterface + Send + Sync>>;
    fn get_client(&self) -> TonClient;

    async fn try_execute(&self, msg: &String, interface_id: &String, abi_version: &str) -> Option<InterfaceResult> {
        let res = Self::execute(self.get_client(), msg, interface_id, self.get_interfaces(), abi_version).await;
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
        abi_version: &str,
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
                let abi = object.get_target_abi(abi_version);
                let (func, args) = decode_msg(client.clone(), body, abi.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                let (answer_id, mut ret_args) = object.call(&func, &args)
                    .await
                    .map_err(|e| format!("interface {}.{} failed: {}", interface_id, func, e))?;
                if abi_version == "2.0" {
                    if let Abi::Json(json_str) = abi {
                        let _ = convert_return_args(json_str.as_str(), &func, &mut ret_args)?;
                    }
                }
                Ok((answer_id, ret_args))
            }
            None => {
                debug!("interface {} not implemented", interface_id);
                Ok((0, json!({})))
            },
        }
    }
}

fn convert_return_args(abi: &str, fname: &str, ret_args: &mut Value) -> Result<(), String> {
    let contract = Contract::load(abi.as_bytes()).map_err(|e| format!("{}", e))?;
    let func = contract
        .function(fname)
        .map_err(|_| format!("function with name '{}' not found", fname))?;
    let output = func
        .outputs
        .iter();
    for val in output {
        let pointer = "";
        bypass_json(pointer, ret_args, val.clone(), ParamType::String)?;
    }
    Ok(())
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

        let iface: Arc<dyn DebotInterface + Send + Sync> = Arc::new(QueryInterface::new(client.clone()));
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
        let string = elem.as_str().ok_or_else(|| format!("array element is invalid: must be string"))?;
        strings.push(string.to_owned());
    }
    Ok(strings)
}