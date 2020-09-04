use serde_json::Value;
use crate::abi::abi::Abi;
use crate::error::ApiResult;
use crate::crypto::boxes::Signing;
use crate::client::ClientContext;

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfEncodeMessage {
    pub message: String,
    pub bytes_to_sign: Option<String>,
}

//--------------------------------------------------------------------------- encode_deploy_message

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfDeployMessage {
    /// contract ABI
    pub abi: Abi,
    /// TVC file encoded with `base64`.
    pub tvc: String,

    /// List of initial values for contract public variables
    pub data: Option<Value>,

    /// Init function name. Default is `constructor`.
    pub function_name: Option<String>,
    /// Header parameters
    pub header: Option<Value>,
    /// Init function input parameters according to ABI.
    pub input: Value,

    /// Signing parameters. If omitted, message will be created unsigned.
    pub signing: Option<Signing>,
    /// Target workchain for destination address. Default is `0`.
    pub workchain_id: Option<i32>,
}

pub fn encode_deploy_message(
    _context: &mut ClientContext,
    params: ParamsOfDeployMessage
) -> ApiResult<ResultOfEncodeMessage> {
    Ok(ResultOfEncodeMessage {
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

pub fn encode_run_message(
    _context: &mut ClientContext,
    params: ParamsOfRunMessage
) -> ApiResult<ResultOfEncodeMessage> {
    Ok(ResultOfEncodeMessage {
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

pub fn encode_with_signature(
    _context: &mut ClientContext,
    params: ParamsOfEncodeWithSignature
) -> ApiResult<ResultOfEncodeWithSignature> {
    Ok(ResultOfEncodeWithSignature {
        message: "".into(),
    })
}
