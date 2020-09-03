use crate::abi::types::{Abi, Signing};
use serde_json::Value;
use crate::abi::abi::Abi;
use crate::error::ApiResult;

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfCreateMessage {
    message: String,
    bytes_to_sign: Option<Stroing>,
}

//--------------------------------------------------------------------------- encode_deploy_message

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfDeployMessage {
    /// contract ABI
    pub abi: Abi,

    /// List of initial values for contract public variables
    pub data: Option<Value>,
    /// Offchain constructor input parameters according to contract ABI.
    pub constructor_input: Option<Value>,

    /// Init function name. Default is `constructor`.
    pub function_name: Option<String>,
    /// Header parameters
    pub header: Option<Value>,
    /// Init function input parameters according to ABI.
    pub input: Value,

    /// TVC file encoded with base64
    pub image: String,
    /// Signing parameters. If omitted, message will be created unsigned.
    pub signing: Option<Signing>,
    /// Target workchain for destination address. Default is `0`.
    pub workchain_id: Option<i32>,
}

pub fn encode_deploy_message(params: ParamsOfDeployMessage) -> ApiResult<ResultOfCreateMessage> {
    Ok(ResultOfCreateMessage {
        message: "".into(),
        bytes_to_sign: None,
    })
}

//--------------------------------------------------------------------------- encode_run_message

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfRunMessage {
    /// contract ABI
    pub abi: Abi,

    /// Contract address
    pub address: String,

    /// Init function name. Default is `constructor`.
    pub function_name: Option<String>,
    /// Header parameters
    pub header: Option<Value>,
    /// Init function input parameters according to ABI.
    pub input: Value,

    /// Signing parameters. If omitted, message will be created unsigned.
    pub signing: Option<Signing>,
}

pub fn encode_run_message(params: ParamsOfRunMessage) -> ApiResult<ResultOfCreateMessage> {
    Ok(ResultOfCreateMessage {
        message: "".into(),
        bytes_to_sign: None,
    })
}

//-------------------------------------------------------------------------- encode_with_signature

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfEncodeWithSignature {
    /// Unsigned message BOC
    pub message: String,

    /// Signature
    pub signature: String,
}

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfEncodeWithSignature {
    pub message: String,
}

pub fn encode_with_signature(params: ParamsOfEncodeWithSignature) -> ApiResult<ResultOfEncodeWithSignature> {
    Ok(ResultOfEncodeWithSignature {
        message: "".into(),
    })
}
