use crate::abi;
use crate::abi::internal::{add_sign_to_message, add_sign_to_message_body, create_tvc_image, try_to_sign_message, resolve_pubkey, update_pubkey};
use crate::abi::{Abi, Error, FunctionHeader, Signer};
use crate::boc::internal::{get_boc_hash, deserialize_cell_from_boc};
use crate::client::ClientContext;
use crate::crypto::internal::decode_public_key;
use crate::encoding::{account_decode, account_encode, hex_decode};
use crate::error::ClientResult;
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;
use ton_abi::Contract;
use ton_block::{MsgAddressInt, CurrencyCollection};
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

    /// Optional public key that can be provided in deploy set in order to substitute one
    /// in TVM file or provided by Signer.
    ///
    /// Public key resolving priority:
    /// 1. Public key from deploy set.
    /// 2. Public key, specified in TVM file.
    /// 3. Public key, provided by Signer.
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
        internal: bool,
    ) -> ClientResult<FunctionCallSet> {
        let contract = Contract::load(abi.as_bytes()).map_err(|x| Error::invalid_json(x))?;
        let header = if internal {
            None
        } else {
            resolve_header(
                self.header.as_ref(),
                pubkey,
                processing_try_index,
                context,
                &contract,
            )?
        };

        let func = match u32::from_str_radix(&self.function_name, 16) {
            Ok(id) => &contract.function_by_id(id, true).map_err(|e| Error::invalid_function_id(&self.function_name, e))?.name,
            Err(_) => &self.function_name,
        }.clone();

        Ok(FunctionCallSet {
            abi: abi.to_string(),
            func,
            header: header.as_ref().map(|x| header_to_string(x)),
            input: self
                .input
                .as_ref()
                .map(|x| x.to_string())
                .unwrap_or("{}".into()),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
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

#[derive(Serialize, Deserialize, ApiType, Default)]
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
        call_set.to_function_call_set(pubkey, processing_try_index, &context, &abi, false)?,
        image,
        workchain,
    )
    .map_err(|err| abi::Error::encode_deploy_message_failed(err))?;
    Ok((unsigned.message, Some(unsigned.data_to_sign), address))
}

fn encode_int_deploy(
    context: std::sync::Arc<ClientContext>,
    abi: &str,
    image: ContractImage,
    workchain_id: i32,
    call_set: &CallSet,
    pubkey: Option<&str>,
    ihr_disabled: bool,
    bounce: bool,
) -> ClientResult<(Vec<u8>, MsgAddressInt)> {
    let address = image.msg_address(workchain_id);
    let message = ton_sdk::Contract::get_int_deploy_message_bytes(
        call_set.to_function_call_set(pubkey, None, &context, &abi, true)?,
        image,
        workchain_id,
        ihr_disabled,
        bounce,
    ).map_err(|err| abi::Error::encode_deploy_message_failed(err))?;

    Ok((message, address))
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

fn encode_empty_int_deploy(
    image: ContractImage,
    workchain_id: i32,
    ihr_disabled: bool,
    bounce: bool,
) -> ClientResult<(Vec<u8>, MsgAddressInt)> {
    let address = image.msg_address(workchain_id);
    let message = ton_sdk::Contract::construct_int_deploy_message_no_constructor(
        image,
        workchain_id,
        ihr_disabled,
        bounce
    ).map_err(|x| abi::Error::encode_deploy_message_failed(x))?;

    Ok((
        ton_sdk::Contract::serialize_message(&message)
            .map_err(|x| abi::Error::encode_deploy_message_failed(x))?
            .0,
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
            let message = ton_sdk::Contract::construct_call_ext_in_message_json(
                address.clone(),
                call_set.to_function_call_set(pubkey, processing_try_index, &context, abi, false)?,
                None,
            )
            .map_err(|err| abi::Error::encode_run_message_failed(err, &call_set.function_name))?;
            (message.serialized_message, None, address)
        }
        _ => {
            let unsigned = ton_sdk::Contract::get_call_message_bytes_for_signing(
                address.clone(),
                call_set.to_function_call_set(pubkey, processing_try_index, &context, abi, false)?,
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
///
/// There is an optional public key can be provided in deploy set in order to substitute one
/// in TVM file.
///
/// Public key resolving priority:
/// 1. Public key from deploy set.
/// 2. Public key, specified in TVM file.
/// 3. Public key, provided by signer.

#[api_function]
pub async fn encode_message(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfEncodeMessage,
) -> ClientResult<ResultOfEncodeMessage> {
    let abi = params.abi.json_string()?;

    let public = params.signer.resolve_public_key(context.clone()).await?;
    let (message, data_to_sign, address) = if let Some(deploy_set) = params.deploy_set {
        let workchain = deploy_set
            .workchain_id
            .unwrap_or(context.config.abi.workchain);
        let mut image = create_tvc_image(
            &context,
            &abi,
            deploy_set.initial_data.as_ref(),
            &deploy_set.tvc,
        ).await?;

        if let Some(tvc_public) = resolve_pubkey(&deploy_set, &image, &public)? {
            image.set_public_key(&decode_public_key(&tvc_public)?)
                .map_err(|err| Error::invalid_tvc_image(err))?;
        }

        let public = required_public_key(public)?;
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

    let (message, data_to_sign) = try_to_sign_message(
        context, &abi, message, data_to_sign, &params.signer
    ).await?;

    Ok(ResultOfEncodeMessage {
        message: base64::encode(&message),
        data_to_sign: data_to_sign.map(|data| base64::encode(&data)),
        address: account_encode(&address),
        message_id: get_boc_hash(&message)?,
    })
}

//------------------------------------------------------------------------ encode_internal_message

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct ParamsOfEncodeInternalMessage {
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

    /// Value in nanograms to be sent with message.
    pub value: String,

    /// Flag of bounceable message. Default is true.
    pub bounce: Option<bool>,

    /// Enable Instant Hypercube Routing for the message. Default is false.
    pub enable_ihr: Option<bool>,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfEncodeInternalMessage {
    /// Message BOC encoded with `base64`.
    pub message: String,

    /// Destination address.
    pub address: String,

    /// Message id.
    pub message_id: String,
}

/// Encodes an internal ABI-compatible message
///
/// Allows to encode deploy and function call messages.
///
/// Use cases include messages of any possible type:
/// - deploy with initial function call (i.e. `constructor` or any other function that is used for some kind
/// of initialization);
/// - deploy without initial function call;
/// - simple function call
///
/// There is an optional public key can be provided in deploy set in order to substitute one
/// in TVM file.
///
/// Public key resolving priority:
/// 1. Public key from deploy set.
/// 2. Public key, specified in TVM file.

#[api_function]
pub async  fn encode_internal_message(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfEncodeInternalMessage,
) -> ClientResult<ResultOfEncodeInternalMessage> {
    let abi = params.abi.json_string()?;

    let ihr_disabled = !params.enable_ihr.unwrap_or(false);
    let bounce = params.bounce.unwrap_or(true);

    let (message, address) = if let Some(deploy_set) = params.deploy_set {
        let workchain_id = deploy_set
            .workchain_id
            .unwrap_or(context.config.abi.workchain);
        let mut image = create_tvc_image(
            &context,
            &abi,
            deploy_set.initial_data.as_ref(),
            &deploy_set.tvc,
        ).await?;

        let public = update_pubkey(&deploy_set, &mut image, &None)?;
        let public = required_public_key(public)?;
        if let Some(call_set) = &params.call_set {
            encode_int_deploy(
                Arc::clone(&context),
                &abi,
                image,
                workchain_id,
                call_set,
                Some(&public),
                ihr_disabled,
                bounce,
            )?
        } else {
            encode_empty_int_deploy(image, workchain_id, ihr_disabled, bounce)?
        }
    } else if let Some(call_set) = &params.call_set {
        let address = params
            .address
            .as_ref()
            .ok_or(abi::Error::required_address_missing_for_encode_message())?;
        let address = account_decode(address)?;

        let message = ton_sdk::Contract::construct_call_int_message_json(
            address.clone(),
            ihr_disabled,
            bounce,
            CurrencyCollection::with_grams(
                u64::from_str(&params.value)
                    .map_err(|err| abi::Error::encode_run_message_failed(err, ""))?
            ),
            call_set.to_function_call_set(None, None, &context, &abi, true)?,
        )
        .map_err(|err| abi::Error::encode_run_message_failed(err, &call_set.function_name))?;

        (message.serialized_message, address)
    } else {
        return Err(abi::Error::missing_required_call_set_for_encode_message());
    };

    Ok(ResultOfEncodeInternalMessage {
        message: base64::encode(&message),
        address: account_encode(&address),
        message_id: get_boc_hash(&message)?,
    })
}

//---------------------------------------------------------------------------- encode_message_body

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
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

#[derive(Serialize, Deserialize, ApiType, Default)]
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
        params.is_internal,
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
            if params.is_internal {
                ton_abi::encode_function_call(
                    abi.clone(),
                    func.clone(),
                    None,
                    call.input,
                    true,
                    None,
                ).map(|body| (body, None))
            } else {
                ton_abi::prepare_function_call_for_sign(
                    abi.clone(),
                    func.clone(),
                    call.header,
                    call.input,
                ).map(|(body, data_to_sign)| (body, Some(data_to_sign)))
            }.map_err(|err| Error::encode_run_message_failed(err, &func))?
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

#[derive(Serialize, Deserialize, ApiType, Default)]
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

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfAttachSignature {
    /// Signed message BOC
    pub message: String,
    /// Message ID
    pub message_id: String,
}

/// Combines `hex`-encoded `signature` with `base64`-encoded `unsigned_message`.
/// Returns signed message encoded in `base64`.
#[api_function]
pub async fn attach_signature(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfAttachSignature,
) -> ClientResult<ResultOfAttachSignature> {
    let (boc, _) = deserialize_cell_from_boc(&context, &params.message, "message").await?;
    let signed = add_sign_to_message(
        &params.abi.json_string()?,
        &hex_decode(&params.signature)?,
        Some(&hex_decode(&params.public_key)?),
        &boc.bytes("message")?,
    )?;
    Ok(ResultOfAttachSignature {
        message: base64::encode(&signed),
        message_id: get_boc_hash(&signed)?,
    })
}

//---------------------------------------------------------------- attach_signature_to_message_body

#[derive(Serialize, Deserialize, ApiType, Default)]
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

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfAttachSignatureToMessageBody {
    pub body: String,
}

///
#[api_function]
pub async fn attach_signature_to_message_body(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfAttachSignatureToMessageBody,
) -> ClientResult<ResultOfAttachSignatureToMessageBody> {
    let (boc, _) = deserialize_cell_from_boc(&context, &params.message, "message body").await?;
    let signed = add_sign_to_message_body(
        &params.abi.json_string()?,
        &hex_decode(&params.signature)?,
        Some(&hex_decode(&params.public_key)?),
        &boc.bytes("message body")?,
    )?;
    Ok(ResultOfAttachSignatureToMessageBody {
        body: base64::encode(&signed),
    })
}
