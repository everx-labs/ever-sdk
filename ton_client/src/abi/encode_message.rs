use crate::abi;
use crate::abi::internal::{add_sign_to_message, add_sign_to_message_body, create_tvc_image, result_of_encode_message, resolve_pubkey};
use crate::abi::{Abi, Error, FunctionHeader, Signer};
use crate::boc::internal::get_boc_hash;
use crate::client::ClientContext;
use crate::crypto::internal::decode_public_key;
use crate::encoding::{account_decode, account_encode, base64_decode, hex_decode};
use crate::error::ClientResult;
use serde_json::Value;
use std::sync::Arc;
use ton_abi::Contract;
use ton_block::MsgAddressInt;
use ton_sdk::{ContractImage, FunctionCallSet};

//--------------------------------------------------------------------------- encode_deploy_message

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct DeploySet {
    /// Content of TVC file encoded in `base64`.
    pub tvc: String,

    /// Target workchain for destination address. Default is `0`.
    pub workchain_id: Option<i32>,

    /// List of initial values for contract's public variables.
    pub initial_data: Option<Value>,

    /// Initial value for contract's public key. Encoded with `hex`.
    pub initial_pubkey: Option<String>,
}

impl DeploySet {
    pub fn some_with_tvc(tvc: String) -> Option<Self> {
        Some(Self {
            tvc,
            workchain_id: None,
            initial_data: None,
            initial_pubkey: None,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct CallSet {
    /// Function name that is being called.
    pub function_name: String,

    /// Function header.
    ///
    /// If an application omits some header parameters required by the
    /// contract's ABI, the library will set the default values for
    /// them.
    pub header: Option<FunctionHeader>,

    /// Function input parameters according to ABI.
    pub input: Option<Value>,
}

impl CallSet {
    pub fn some_with_function(function: &str) -> Option<Self> {
        Some(Self {
            function_name: function.into(),
            header: None,
            input: None,
        })
    }
    pub fn some_with_function_and_input(function: &str, input: Value) -> Option<Self> {
        Some(Self {
            function_name: function.into(),
            input: Some(input),
            header: None,
        })
    }
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
) -> ClientResult<Option<FunctionHeader>> {
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
                    config.message_expiration_timeout,
                    config.message_expiration_timeout_grow_factor,
                    processing_try_index.unwrap_or(0),
                );
                ((now + timeout as u64) / 1000) as u32
            }))
        } else {
            None
        },
        pubkey: if required("pubkey") {
            header
                .map_or(None, |x| x.pubkey.clone())
                .or(pubkey.map(|x| x.to_string()))
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
    ) -> ClientResult<FunctionCallSet> {
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

    /// Target address the message will be sent to.
    ///
    /// Must be specified in case of non-deploy message.
    pub address: Option<String>,

    /// Deploy parameters.
    ///
    /// Must be specified in case of deploy message.
    pub deploy_set: Option<DeploySet>,

    /// Function call parameters.
    ///
    /// Must be specified in case of non-deploy message.
    ///
    /// In case of deploy message it is optional and contains parameters
    /// of the functions that will to be called upon deploy transaction.
    pub call_set: Option<CallSet>,

    /// Signing parameters.
    pub signer: Signer,

    /// Processing try index.
    ///
    /// Used in message processing with retries (if contract's ABI includes "expire" header).
    ///
    /// Encoder uses the provided try index to calculate message
    /// expiration time. The 1st message expiration time is specified in
    /// Client config.
    ///
    /// Expiration timeouts will grow with every retry.
    /// Retry grow factor is set in Client config:
    /// <.....add config parameter with default value here>
    ///
    /// Default value is 0.
    pub processing_try_index: Option<u8>,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfEncodeMessage {
    /// Message BOC encoded with `base64`.
    pub message: String,

    /// Optional data to be signed encoded in `base64`.
    ///
    /// Returned in case of `Signer::External`. Can be used for external
    /// message signing. Is this case you need to use this data to create signature and
    /// then produce signed message using `abi.attach_signature`.
    pub data_to_sign: Option<String>,

    /// Destination address.
    pub address: String,

    /// Message id.
    pub message_id: String,
}

fn required_public_key(public_key: Option<String>) -> ClientResult<String> {
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
) -> ClientResult<(Vec<u8>, Option<Vec<u8>>, MsgAddressInt)> {
    let address = image.msg_address(workchain);
    let unsigned = ton_sdk::Contract::get_deploy_message_bytes_for_signing(
        call_set.to_function_call_set(pubkey, processing_try_index, &context, &abi)?,
        image,
        workchain,
    )
    .map_err(|err| abi::Error::encode_deploy_message_failed(err))?;
    Ok((unsigned.message, Some(unsigned.data_to_sign), address))
}

fn encode_empty_deploy(
    image: ContractImage,
    workchain: i32,
) -> ClientResult<(Vec<u8>, Option<Vec<u8>>, MsgAddressInt)> {
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
) -> ClientResult<(Vec<u8>, Option<Vec<u8>>, MsgAddressInt)> {
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
            )
            .map_err(|err| abi::Error::encode_run_message_failed(err, &call_set.function_name))?;
            (message.serialized_message, None, address)
        }
        _ => {
            let unsigned = ton_sdk::Contract::get_call_message_bytes_for_signing(
                address.clone(),
                call_set.to_function_call_set(pubkey, processing_try_index, &context, abi)?,
            )
            .map_err(|err| abi::Error::encode_run_message_failed(err, &call_set.function_name))?;

            (unsigned.message, Some(unsigned.data_to_sign), address)
        }
    })
}

/// Encodes an ABI-compatible message
///
/// Allows to encode deploy and function call messages,
/// both signed and unsigned.
///
/// Use cases include messages of any possible type:
/// - deploy with initial function call (i.e. `constructor` or any other function that is used for some kind
/// of initialization);
/// - deploy without initial function call;
/// - signed/unsigned + data for signing.
///
/// `Signer` defines how the message should or shouldn't be signed:
///
/// `Signer::None` creates an unsigned message. This may be needed in case of some public methods,
/// that do not require authorization by pubkey.
///
/// `Signer::External` takes public key and returns `data_to_sign` for later signing.
/// Use `attach_signature` method with the result signature to get the signed message.
///
/// `Signer::Keys` creates a signed message with provided key pair.
///
/// [SOON] `Signer::SigningBox` Allows using a special interface to implement signing
/// without private key disclosure to SDK. For instance, in case of using a cold wallet or HSM,
/// when application calls some API to sign data.

#[api_function]
pub async fn encode_message(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfEncodeMessage,
) -> ClientResult<ResultOfEncodeMessage> {
    let abi = params.abi.json_string()?;

    let deploy_set = params.deploy_set.as_ref().map(
        |deploy_set|
            create_tvc_image(
                &abi,
                deploy_set.initial_data.as_ref(),
                &deploy_set.tvc,
            ).map(|image| (deploy_set, image))
    ).transpose()?;

    let public = resolve_pubkey(&context, &deploy_set, &params.signer).await?;

    let (message, data_to_sign, address) = if let Some((deploy_set, mut image)) = deploy_set {
        let public = required_public_key(public)?;
        image.set_public_key(&decode_public_key(&public)?)
            .map_err(|err| Error::invalid_tvc_image(err))?;

        let workchain = deploy_set
            .workchain_id
            .unwrap_or(context.config.abi.workchain);
        if let Some(call_set) = &params.call_set {
            encode_deploy(
                context.clone(),
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
            context.clone(),
            &params,
            &abi,
            call_set,
            public.as_ref().map(|x| x.as_str()),
            params.processing_try_index,
        )?
    } else {
        return Err(abi::Error::missing_required_call_set_for_encode_message());
    };

    let (message, data_to_sign) = result_of_encode_message(
        context, &abi, message, data_to_sign, &params.signer
    ).await?;

    Ok(ResultOfEncodeMessage {
        message: base64::encode(&message),
        data_to_sign,
        address: account_encode(&address),
        message_id: get_boc_hash(&message)?,
    })
}

//---------------------------------------------------------------------------- encode_message_body

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
pub struct ParamsOfEncodeMessageBody {
    /// Contract ABI.
    pub abi: Abi,

    /// Function call parameters.
    ///
    /// Must be specified in non deploy message.
    ///
    /// In case of deploy message contains parameters of constructor.
    pub call_set: CallSet,

    /// True if internal message body must be encoded.
    pub is_internal: bool,

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
pub struct ResultOfEncodeMessageBody {
    /// Message body BOC encoded with `base64`.
    pub body: String,

    /// Optional data to sign. Encoded with `base64`.
    ///
    /// Presents when `message` is unsigned. Can be used for external
    /// message signing. Is this case you need to sing this data and
    /// produce signed message using `abi.attach_signature`.
    pub data_to_sign: Option<String>,
}

/// Encodes message body according to ABI function call.
#[api_function]
pub async fn encode_message_body(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfEncodeMessageBody,
) -> ClientResult<ResultOfEncodeMessageBody> {
    let abi = params.abi.json_string()?;

    let public = params.signer.resolve_public_key(context.clone()).await?;
    let call = params.call_set.to_function_call_set(
        public.as_ref().map(|x| x.as_str()),
        params.processing_try_index,
        &context,
        &abi,
    )?;
    let func = call.func.clone();
    let (body, data_to_sign) = match params.signer {
        Signer::None => {
            let body = ton_abi::encode_function_call(
                abi.clone(),
                func.clone(),
                call.header,
                call.input.clone(),
                params.is_internal,
                None,
            )
            .map_err(|err| Error::encode_run_message_failed(err, &func))?;
            (body, None)
        }
        _ => {
            let (body, data_to_sign) = ton_abi::prepare_function_call_for_sign(
                abi.clone(),
                func.clone(),
                call.header,
                call.input,
            )
            .map_err(|err| Error::encode_run_message_failed(err, &func))?;
            (body, Some(data_to_sign))
        }
    };
    let body: Vec<u8> = ton_types::serialize_toc(
        &body
            .clone()
            .into_cell()
            .map_err(|err| Error::encode_run_message_failed(err, &func))?,
    )
    .map_err(|err| Error::encode_run_message_failed(err, &func))?;
    if let Some(unsigned) = &data_to_sign {
        if let Some(signature) = params.signer.sign(context.clone(), unsigned).await? {
            let pubkey = public
                .map(|string| hex_decode(&string))
                .transpose()?;
            let body = add_sign_to_message_body(
                &abi,
                &signature,
                pubkey.as_ref().map(|vec| vec.as_slice()),
                &body,
            )?;
            return Ok(ResultOfEncodeMessageBody {
                body: base64::encode(&body),
                data_to_sign: None,
            });
        }
    }
    Ok(ResultOfEncodeMessageBody {
        body: base64::encode(&body),
        data_to_sign: data_to_sign.map(|x| base64::encode(&x)),
    })
}

//------------------------------------------------------------------------------- attach_signature

#[derive(Serialize, Deserialize, ApiType)]
pub struct ParamsOfAttachSignature {
    /// Contract ABI
    pub abi: Abi,

    /// Public key encoded in `hex`.
    pub public_key: String,

    /// Unsigned message BOC encoded in `base64`.
    pub message: String,

    /// Signature encoded in `hex`.
    pub signature: String,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfAttachSignature {
    /// Signed message BOC
    pub message: String,
    /// Message ID
    pub message_id: String,
}

/// Combines `hex`-encoded `signature` with `base64`-encoded `unsigned_message`.
/// Returns signed message encoded in `base64`.
#[api_function]
pub fn attach_signature(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfAttachSignature,
) -> ClientResult<ResultOfAttachSignature> {
    let signed = add_sign_to_message(
        &params.abi.json_string()?,
        &hex_decode(&params.signature)?,
        Some(&hex_decode(&params.public_key)?),
        &base64_decode(&params.message)?,
    )?;
    Ok(ResultOfAttachSignature {
        message: base64::encode(&signed),
        message_id: get_boc_hash(&signed)?,
    })
}

//---------------------------------------------------------------- attach_signature_to_message_body

#[derive(Serialize, Deserialize, ApiType)]
pub struct ParamsOfAttachSignatureToMessageBody {
    /// Contract ABI
    pub abi: Abi,

    /// Public key. Must be encoded with `hex`.
    pub public_key: String,

    /// Unsigned message body BOC. Must be encoded with `base64`.
    pub message: String,

    /// Signature. Must be encoded with `hex`.
    pub signature: String,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfAttachSignatureToMessageBody {
    pub body: String,
}

///
#[api_function]
pub fn attach_signature_to_message_body(
    _context: std::sync::Arc<ClientContext>,
    params: ParamsOfAttachSignatureToMessageBody,
) -> ClientResult<ResultOfAttachSignatureToMessageBody> {
    let signed = add_sign_to_message_body(
        &params.abi.json_string()?,
        &hex_decode(&params.signature)?,
        Some(&hex_decode(&params.public_key)?),
        &base64_decode(&params.message)?,
    )?;
    Ok(ResultOfAttachSignatureToMessageBody {
        body: base64::encode(&signed),
    })
}
