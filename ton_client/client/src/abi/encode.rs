use crate::abi::internal::{
    add_sign_to_message, create_tvc_image, resolve_abi, result_of_encode_message,
};
use crate::abi::{Abi, MessageSigning};
use crate::client::ClientContext;
use crate::encoding::{account_decode, base64_decode, hex_decode};
use crate::error::{ApiError, ApiResult};
use serde_json::Value;
use ton_sdk::FunctionCallSet;
use crate::abi::defaults::DEFAULT_WORKCHAIN;

//--------------------------------------------------------------------------- encode_deploy_message

#[derive(Serialize, Deserialize, Clone, Debug, TypeInfo)]
pub struct DeploySet {
    /// Target workchain for destination address. Default is `0`.
    pub workchain_id: Option<i32>,
    /// Content of TVC file. Must be encoded with `base64`.
    pub tvc: String,
    /// List of initial values for contract public variables.
    pub initial_data: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, TypeInfo)]
pub struct CallSet {
    /// Function name.
    pub function_name: String,
    /// Header parameters.
    pub header: Option<Value>,
    /// Init function input parameters according to ABI.
    pub input: Option<Value>,
}

fn to_function_call_set(call_set: &Option<CallSet>, abi: &str) -> FunctionCallSet {
    if let Some(call_set) = call_set.as_ref() {
        FunctionCallSet {
            abi: abi.to_string(),
            func: call_set.function_name.clone(),
            header: call_set.header.as_ref().map(|x| x.to_string()),
            input: call_set.input.as_ref().map(|x| x.to_string()).unwrap_or("{}".into()),
        }
    } else {
        FunctionCallSet {
            abi: abi.to_string(),
            func: "constructor".into(),
            header: None,
            input: "{}".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TypeInfo)]
pub struct ParamsOfEncodeMessage {
    /// Contract ABI.
    pub abi: Abi,
    /// Contract address.
    /// Must be specified in case of run message.
    pub address: Option<String>,
    /// Deploy parameters.
    /// Must be specified in case of deploy message.
    pub deploy_set: Option<DeploySet>,
    /// Function call parameters.
    /// Must be specified in run message.
    /// In case of deploy message contains parameters of constructor.
    pub call_set: Option<CallSet>,
    /// Signing parameters.
    pub signing: MessageSigning,
}

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfEncodeMessage {
    /// Message BOC encoded with `base64`.
    pub message: String,
    /// Optional data to sign. Encoded with `base64`.
    /// Presents when `message` is unsigned.
    /// Can be used for external message signing.
    /// Is this case you need to sing this data and
    /// produce signed message using `abi.attach_signature`.
    pub data_to_sign: Option<String>,
}

fn required_public_key(public_key: Option<String>) -> ApiResult<String> {
    if let Some(public_key) = public_key {
        Ok(public_key)
    } else {
        Err(ApiError::contracts_create_deploy_message_failed(
            "Public key doesn't provided.",
        ))
    }
}

pub fn encode_message(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfEncodeMessage,
) -> ApiResult<ResultOfEncodeMessage> {
    let abi = resolve_abi(&params.abi)?;
    trace!("-> abi.encode_deploy_message({:?})", params.clone());

    let unsigned = if let Some(deploy_set) = params.deploy_set {
        let workchain = deploy_set.workchain_id.unwrap_or(DEFAULT_WORKCHAIN);
        let public = required_public_key(params.signing.resolve_public_key()?)?;
        let image = create_tvc_image(
            &abi,
            deploy_set.initial_data.as_ref(),
            &deploy_set.tvc,
            &public,
        )?;
        //TODO: let address = account_encode(&image.msg_address(workchain));
        ton_sdk::Contract::get_deploy_message_bytes_for_signing(
            to_function_call_set(&params.call_set, &abi),
            image,
            workchain,
            &context.config.abi,
            None, //TODO: params.try_index,
        )
        .map_err(|err| ApiError::contracts_create_deploy_message_failed(err))?
    } else if let Some(call_set) = &params.call_set {
        let address = &params
            .address
            .ok_or(ApiError::abi_required_address_missing_for_encode_message())?;
        ton_sdk::Contract::get_call_message_bytes_for_signing(
            account_decode(address)?,
            to_function_call_set(&params.call_set, &abi),
            &context.config.abi,
            None,
        )
        .map_err(|err| {
            ApiError::contracts_create_run_message_failed(err, &call_set.function_name)
        })?
    } else {
        return Err(ApiError::abi_missing_required_call_set_for_encode_message());
    };

    trace!("<-");
    let (message, data_to_sign) = result_of_encode_message(
        &abi,
        &unsigned.message,
        &unsigned.data_to_sign,
        &params.signing,
    )?;
    Ok(ResultOfEncodeMessage {
        message,
        data_to_sign,
    })
}

//------------------------------------------------------------------------------- attach_signature

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ParamsOfAttachSignature {
    /// Contract ABI
    pub abi: Abi,

    /// Public key. Must be encoded with `hex`.
    pub public_key: String,

    /// Unsigned message BOC. Must be encoded with `base64`.
    pub message: String,

    /// Signature. Must be encoded with `hex`.
    pub signature: String,
}

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfAttachSignature {
    pub message: String,
}

pub fn attach_signature(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfAttachSignature,
) -> ApiResult<ResultOfAttachSignature> {
    let signed = add_sign_to_message(
        &resolve_abi(&params.abi)?,
        &hex_decode(&params.signature)?,
        Some(&hex_decode(&params.public_key)?),
        &base64_decode(&params.message)?,
    )?;
    Ok(ResultOfAttachSignature {
        message: base64::encode(&signed),
    })
}
