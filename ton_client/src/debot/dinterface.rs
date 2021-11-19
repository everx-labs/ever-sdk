use super::base64_interface::Base64Interface;
use super::hex_interface::HexInterface;
use super::sdk_interface::SdkInterface;
use super::network_interface::NetworkInterface;
use super::query_interface::QueryInterface;
use super::JsonValue;
use crate::abi::{Abi};
use crate::boc::{parse_message, ParamsOfParse};
use crate::debot::TonClient;
use crate::encoding::decode_abi_number;
use num_traits::cast::NumCast;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
pub type InterfaceResult = Result<(u32, Value), String>;
use ton_abi::{Contract, Param, ParamType};
use ton_types::SliceData;

use crate::boc::internal::deserialize_cell_from_boc;
use ton_sdk::AbiContract;
use ton_abi::token::Detokenizer;

async fn decode_msg(
    client: TonClient,
    msg_body: String,
    abi: Abi,
) -> Result<(String, Value), String> {
    let abi = abi.json_string().map_err(|e| format!("invalid json: {}", e))?;
    let abi = AbiContract::load(abi.as_bytes()).map_err(|e| format!("invalid json: {}", e))?;
    let (_, body) = deserialize_cell_from_boc(&client, &msg_body, "message body").await.unwrap();
    let body: SliceData = body.into();
    let input = abi.decode_input(body.clone(), true).map_err(|e| format!("can't decode input: {}", e))?;
    let value = Detokenizer::detokenize_to_json_value(&input.tokens).map_err(|e| format!("detokenize error: {}", e))?;
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
                println!("body {}", body);
                let abi = object.get_target_abi(abi_version);
                let (func, args) = decode_msg(client.clone(), body, abi.clone()).await?;
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
        bypass_return_args(pointer, ret_args, val.clone())?;
    }
    Ok(())
}

fn bypass_return_args(top_pointer: &str, obj: &mut Value, p: Param) -> Result<(), String> {
    let pointer = format!("{}/{}", top_pointer, p.name);
    match p.kind {
        ParamType::String => {
            string_to_hex(obj, &pointer).map_err(|e| format!("{}: \"{}\"", e, p.name))?;
        }
        ParamType::Tuple(params) => {
            for p in params {
                bypass_return_args(&pointer, obj, p)?;
            }
        }
        ParamType::Array(ref elem_type) => {
            let elem_count = obj
                .pointer(&pointer)
                .ok_or_else(|| format!("\"{}\" not found", pointer))?
                .as_array()
                .ok_or_else(|| String::from("Failed to retrieve an array"))?
                .len();
            for i in 0..elem_count {
                bypass_return_args(
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
                .ok_or_else(|| String::from("Failed to retrieve an object"))?
                .keys()
                .map(|k| k.clone())
                .collect();
            for key in keys {
                bypass_return_args(&pointer, obj, Param::new(key.as_str(), (**value).clone()))?;
            }
        }
        _ => (),
    }
    Ok(())
}

fn string_to_hex(obj: &mut Value, pointer: &str) -> Result<(), String> {
    let val_str = obj
        .pointer(pointer)
        .ok_or_else(|| format!("argument not found"))?
        .as_str()
        .ok_or_else(|| format!("argument not a string"))?;
    *obj.pointer_mut(pointer).unwrap() = json!(hex::encode(val_str));
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