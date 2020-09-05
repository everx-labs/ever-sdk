use serde_json::Value;
use crate::abi::abi::Abi;
use crate::error::{ApiResult, ApiError};
use crate::crypto::boxes::Signing;
use crate::client::ClientContext;
use ton_sdk::{FunctionCallSet};
use crate::crypto::internal::{decode_public_key, sign_using_keys};
use ed25519_dalek::{Keypair, PublicKey};
use crate::contracts::deploy::create_image;

const DEFAULT_WORKCHAIN: i32 = 0;
const DEFAULT_CONSTRUCTOR_FUNCTION_NAME: &str = "constructor";

#[derive(Serialize, Deserialize, TypeInfo)]
pub struct ResultOfEncodeMessage {
    /// Message BOC encoded with `base64`.
    pub message: String,
    /// Optional data to sign. Presents when `message` is unsigned.
    /// Can be used to external message signing.
    /// Is this case you need to sing this data and
    /// produce signed message using `abi.attach_signature`.
    pub data_to_sign: Option<String>,
}

//--------------------------------------------------------------------------- encode_deploy_message

#[derive(Serialize, Deserialize, Clone, Debug, TypeInfo)]
pub struct ParamsOfDeployMessage {
    /// contract ABI
    pub abi: Abi,
    /// TVC file encoded with `base64`.
    pub tvc: String,

    pub public_key: Option<String>,

    /// List of initial values for contract public variables
    pub initial_data: Option<Value>,

    /// Init function name. Default is `constructor`.
    pub function_name: Option<String>,
    /// Header parameters
    pub header: Option<Value>,
    /// Init function input parameters according to ABI.
    pub input: Option<Value>,

    /// Signing parameters. If omitted, message will be created unsigned.
    pub signing: Option<Signing>,
    /// Target workchain for destination address. Default is `0`.
    pub workchain_id: Option<i32>,
}

fn resolve_abi(abi: &Abi) -> ApiResult<Value> {
    if let Abi::Value(value) = abi {
        Ok(value.clone())
    } else {
        Err(ApiError::contracts_create_deploy_message_failed("Abi handle doesn't supported yet"))
    }
}

fn resolve_keys(signing: &Option<Signing>) -> ApiResult<Option<ed25519_dalek::Keypair>> {
    if let Some(signing) = signing {
        if let Signing::Keys(keys) = signing {
            Ok(Some(keys.decode()?))
        } else {
            Err(ApiError::contracts_create_deploy_message_failed("Abi handle doesn't supported yet"))
        }
    } else {
        Ok(None)
    }
}

fn resolve_public_key(keys: &Option<Keypair>, public_key: &Option<String>) -> ApiResult<PublicKey> {
    if let Some(public_key) = public_key {
        decode_public_key(&public_key)
    } else if let Some(keys) = keys {
        Ok(keys.public.clone())
    } else {
        Err(ApiError::contracts_create_deploy_message_failed("Public key doesn't provided."))
    }
}

pub fn encode_deploy_message(
    context: &mut ClientContext,
    params: ParamsOfDeployMessage,
) -> ApiResult<ResultOfEncodeMessage> {
    let abi = resolve_abi(&params.abi)?;
    trace!("-> abi.encode_deploy_message({:?})", params.clone());

    let workchain = params.workchain_id.unwrap_or(DEFAULT_WORKCHAIN);
    let keys = resolve_keys(&params.signing)?;
    let public = resolve_public_key(&keys, &params.public_key)?;

    let image = create_image(
        &abi,
        params.initial_data.as_ref(),
        &params.tvc,
        &public,
    )?;
    //TODO: let address = account_encode(&image.msg_address(workchain));
    let unsigned = ton_sdk::Contract::get_deploy_message_bytes_for_signing(
        FunctionCallSet {
            abi: abi.to_string(),
            func: params.function_name.unwrap_or(DEFAULT_CONSTRUCTOR_FUNCTION_NAME.into()).clone(),
            header: params.header.map(|x| x.to_string()).clone(),
            input: params.input.map(|x| x.to_string()).unwrap_or("{}".into()),
        },
        image,
        workchain,
        Some(context.get_client()?.timeouts()),
        None, //TODO: params.try_index,
    ).map_err(|err| ApiError::contracts_create_deploy_message_failed(err))?;

    trace!("<-");
    let (message, data_to_sign) = if let Some(keys) = &keys {
        let signature = sign_using_keys(&unsigned.data_to_sign, &keys)?;
        let signed = add_sign_to_message(
            &abi.to_string(),
            &signature,
            Some(public.as_bytes().as_ref()),
            &unsigned.message,
        )?;
        (signed, None)
    } else {
        (unsigned.message, Some(unsigned.data_to_sign))
    };

    Ok(ResultOfEncodeMessage {
        message: base64::encode(&message),
        data_to_sign: data_to_sign.map(|x|base64::encode(&x)),
    })
}

/// Combines `hex` encoded `signature` with `base64` encoded `unsigned_message`.
/// Returns signed message encoded with `base64`.
fn add_sign_to_message(
    abi: &String,
    signature: &[u8],
    public_key: Option<&[u8]>,
    unsigned_message: &[u8],
) -> ApiResult<Vec<u8>> {
    let signed = ton_sdk::Contract::add_sign_to_message(
        abi.clone(),
        signature,
        public_key,
        unsigned_message,
    ).map_err(|err| ApiError::contracts_encode_message_with_sign_failed(err))?;
    Ok(signed.serialized_message)
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
    _params: ParamsOfRunMessage,
) -> ApiResult<ResultOfEncodeMessage> {
    Ok(ResultOfEncodeMessage {
        message: "".into(),
        data_to_sign: None,
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
    _params: ParamsOfEncodeWithSignature,
) -> ApiResult<ResultOfEncodeWithSignature> {
    Ok(ResultOfEncodeWithSignature {
        message: "".into(),
    })
}
