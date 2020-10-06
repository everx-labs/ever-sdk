use crate::abi;
use crate::abi::internal::{
    add_sign_to_message, create_tvc_image, resolve_abi, result_of_encode_message,
};
use crate::abi::{Abi, Error, FunctionHeader, Signer, DEFAULT_WORKCHAIN};
use crate::boc::get_boc_hash;
use crate::client::ClientContext;
use crate::encoding::{account_decode, account_encode, base64_decode, hex_decode};
use crate::error::ApiResult;
use serde_json::Value;
use std::sync::Arc;
use ton_abi::Contract;
use ton_block::{MsgAddressInt, Serializable};
use ton_sdk::{ContractImage, FunctionCallSet};

//--------------------------------------------------------------------------- encode_deploy_message

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub struct DeploySet {
    /// Content of TVC file. Must be encoded with `base64`.
    pub tvc: String,

    /// Target workchain for destination address. Default is `0`.
    pub workchain_id: Option<i32>,

    /// List of initial values for contract's public variables.
    pub initial_data: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub struct CallSet {
    /// Function name.
    pub function_name: String,

    /// Function header.
    ///
    /// If an application omit some parameters required by the
    /// contract's ABI, the library will set the default values for
    /// it.
    pub header: Option<FunctionHeader>,

    /// Function input according to ABI.
    pub input: Option<Value>,
}

fn calc_timeout(timeout: u32, grow_rate: f32, processing_try_index: u8) -> u32 {
    (timeout as f64 * grow_rate.powi(processing_try_index as i32) as f64) as u32
}

fn resolve_header(
    header: Option<&FunctionHeader>,
    pubkey: Option<&str>,
    processing_try_index: Option<u8>,
    context: &Arc<ClientContext>,
    abi: &Contract,
) -> ApiResult<Option<FunctionHeader>> {
    if abi.header().len() == 0 {
        return Ok(None);
    }
    let now = context.env.now_ms();
    let required = |name: &str| abi.header().iter().find(|x| x.name == name).is_some();
    Ok(Some(FunctionHeader {
        time: if required("time") {
            Some(header.map_or(None, |x| x.time).unwrap_or_else(|| now))
        } else {
            None
        },
        expire: if required("expire") {
            Some(header.map_or(None, |x| x.expire).unwrap_or_else(|| {
                let config = &context.config.abi;
                let timeout = calc_timeout(
                    config.message_expiration_timeout(),
                    config.message_expiration_timeout_grow_factor(),
                    processing_try_index.unwrap_or(0),
                );
                ((now + timeout as u64) / 1000) as u32
            }))
        } else {
            None
        },
        pubkey: if required("pubkey") {
            Some(
                header
                    .map_or(None, |x| x.pubkey.clone())
                    .or(pubkey.map(|x| x.to_string()))
                    .ok_or(Error::required_public_key_missing_for_function_header())?,
            )
        } else {
            None
        },
    }))
}

fn header_to_string(header: &FunctionHeader) -> String {
    let mut values = Vec::<String>::new();
    if let Some(time) = header.time {
        values.push(format!("\"time\": {}", time));
    }
    if let Some(expire) = header.expire {
        values.push(format!("\"expire\": {}", expire));
    }
    if let Some(pubkey) = &header.pubkey {
        values.push(format!("\"pubkey\": \"{}\"", pubkey));
    }
    format!("{{{}}}", values.join(","))
}

impl CallSet {
    fn to_function_call_set(
        &self,
        pubkey: Option<&str>,
        processing_try_index: Option<u8>,
        context: &Arc<ClientContext>,
        abi: &str,
    ) -> ApiResult<FunctionCallSet> {
        let contract = Contract::load(abi.as_bytes()).map_err(|x| Error::invalid_json(x))?;
        let header = resolve_header(
            self.header.as_ref(),
            pubkey,
            processing_try_index,
            context,
            &contract,
        )?;
        Ok(FunctionCallSet {
            abi: abi.to_string(),
            func: self.function_name.clone(),
            header: header.as_ref().map(|x| header_to_string(x)),
            input: self
                .input
                .as_ref()
                .map(|x| x.to_string())
                .unwrap_or("{}".into()),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub struct ParamsOfEncodeMessage {
    /// Contract ABI.
    pub abi: Abi,

    /// Contract address.
    ///
    /// Must be specified in case of non deploy message.
    pub address: Option<String>,

    /// Deploy parameters.
    ///
    /// Must be specified in case of deploy message.
    pub deploy_set: Option<DeploySet>,

    /// Function call parameters.
    ///
    /// Must be specified in non deploy message.
    ///
    /// In case of deploy message contains parameters of constructor.
    pub call_set: Option<CallSet>,

    /// Signing parameters.
    pub signer: Signer,

    /// Processing try index.
    ///
    /// Used in message processing with retries.
    ///
    /// Encoder uses the provided try index to calculate message
    /// expiration time.
    ///
    /// Expiration timeouts will grow with every retry.
    ///
    /// Default value is 0.
    pub processing_try_index: Option<u8>,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfEncodeMessage {
    /// Message BOC encoded with `base64`.
    pub message: String,

    /// Optional data to sign. Encoded with `base64`.
    ///
    /// Presents when `message` is unsigned. Can be used for external
    /// message signing. Is this case you need to sing this data and
    /// produce signed message using `abi.attach_signature`.
    pub data_to_sign: Option<String>,

    /// Destination address.
    pub address: String,

    /// Message id.
    pub message_id: String,
}

fn required_public_key(public_key: Option<String>) -> ApiResult<String> {
    if let Some(public_key) = public_key {
        Ok(public_key)
    } else {
        Err(abi::Error::encode_deploy_message_failed(
            "Public key doesn't provided.",
        ))
    }
}

fn encode_deploy(
    context: std::sync::Arc<ClientContext>,
    abi: &str,
    image: ContractImage,
    workchain: i32,
    call_set: &CallSet,
    pubkey: Option<&str>,
    processing_try_index: Option<u8>,
) -> ApiResult<(Vec<u8>, Option<Vec<u8>>, MsgAddressInt)> {
    let address = image.msg_address(workchain);
    let unsigned = ton_sdk::Contract::get_deploy_message_bytes_for_signing(
        call_set.to_function_call_set(pubkey, processing_try_index, &context, &abi)?,
        image,
        workchain,
        &context.config.abi,
        processing_try_index,
    )
    .map_err(|err| abi::Error::encode_deploy_message_failed(err))?;
    Ok((unsigned.message, Some(unsigned.data_to_sign), address))
}

fn encode_empty_deploy(
    image: ContractImage,
    workchain: i32,
) -> ApiResult<(Vec<u8>, Option<Vec<u8>>, MsgAddressInt)> {
    let address = image.msg_address(workchain);
    let message = ton_sdk::Contract::construct_deploy_message_no_constructor(image, workchain)
        .map_err(|x| abi::Error::encode_deploy_message_failed(x))?;

    Ok((
        ton_sdk::Contract::serialize_message(&message)
            .map_err(|x| abi::Error::encode_deploy_message_failed(x))?
            .0,
        None,
        address,
    ))
}

fn encode_run(
    context: std::sync::Arc<ClientContext>,
    params: &ParamsOfEncodeMessage,
    abi: &str,
    call_set: &CallSet,
    pubkey: Option<&str>,
    processing_try_index: Option<u8>,
) -> ApiResult<(Vec<u8>, Option<Vec<u8>>, MsgAddressInt)> {
    let address = params
        .address
        .as_ref()
        .ok_or(abi::Error::required_address_missing_for_encode_message())?;
    let address = account_decode(address)?;
    Ok(match params.signer {
        Signer::None => {
            let message = ton_sdk::Contract::construct_call_message_json(
                address.clone(),
                call_set.to_function_call_set(pubkey, processing_try_index, &context, abi)?,
                false,
                None,
                &context.config.abi,
                processing_try_index,
            )
            .map_err(|err| abi::Error::encode_run_message_failed(err, &call_set.function_name))?;
            (message.serialized_message, None, address)
        }
        _ => {
            let unsigned = ton_sdk::Contract::get_call_message_bytes_for_signing(
                address.clone(),
                call_set.to_function_call_set(pubkey, processing_try_index, &context, abi)?,
                &context.config.abi,
                processing_try_index,
            )
            .map_err(|err| abi::Error::encode_run_message_failed(err, &call_set.function_name))?;

            (unsigned.message, Some(unsigned.data_to_sign), address)
        }
    })
}

#[api_function]
pub async fn encode_message(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfEncodeMessage,
) -> ApiResult<ResultOfEncodeMessage> {
    let abi = resolve_abi(&params.abi)?;

    let public = params.signer.resolve_public_key()?;
    let (message, data_to_sign, address) = if let Some(deploy_set) = params.deploy_set {
        let workchain = deploy_set.workchain_id.unwrap_or(DEFAULT_WORKCHAIN);
        let public = required_public_key(public)?;
        let image = create_tvc_image(
            &abi,
            deploy_set.initial_data.as_ref(),
            &deploy_set.tvc,
            &public,
        )?;
        if let Some(call_set) = &params.call_set {
            encode_deploy(
                context,
                &abi,
                image,
                workchain,
                call_set,
                Some(&public),
                params.processing_try_index,
            )?
        } else {
            encode_empty_deploy(image, workchain)?
        }
    } else if let Some(call_set) = &params.call_set {
        encode_run(
            context,
            &params,
            &abi,
            call_set,
            public.as_ref().map(|x| x.as_str()),
            params.processing_try_index,
        )?
    } else {
        return Err(abi::Error::missing_required_call_set_for_encode_message());
    };

    let (message, data_to_sign) =
        result_of_encode_message(&abi, message, data_to_sign, &params.signer)?;
    Ok(ResultOfEncodeMessage {
        message: base64::encode(&message),
        data_to_sign,
        address: account_encode(&address),
        message_id: get_boc_hash(&message)?,
    })
}

//------------------------------------------------------------------------------- attach_signature

#[derive(Serialize, Deserialize, ApiType)]
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

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfAttachSignature {
    pub message: String,
    pub message_id: String,
}

#[api_function]
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
        message_id: get_boc_hash(&signed)?,
    })
}
