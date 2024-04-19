use crate::abi::internal::{
    add_sign_to_message, add_sign_to_message_body, create_tvc_image, try_to_sign_message,
    update_pubkey,
};
use crate::abi::{Abi, Error, FunctionHeader, Signer};
use crate::boc::internal::{deserialize_cell_from_boc, get_boc_hash};
use crate::boc::tvc::{resolve_state_init_cell, state_init_with_code};
use crate::client::ClientContext;
use crate::encoding::{account_decode, account_encode, decode_abi_number, hex_decode};
use crate::error::ClientResult;
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;
use ever_abi::Contract;
use ever_block::{CurrencyCollection, MsgAddressInt};
use ton_sdk::{ContractImage, FunctionCallSet};
use ever_block::Cell;

use super::types::extend_data_to_sign;

//--------------------------------------------------------------------------- encode_deploy_message

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct DeploySet {
    /// Content of TVC file encoded in `base64`.
    /// For compatibility reason this field can contain an encoded  `StateInit`.
    pub tvc: Option<String>,

    /// Contract code BOC encoded with base64.
    pub code: Option<String>,

    /// State init BOC encoded with base64.
    pub state_init: Option<String>,

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
    /// 
    /// Applicable only for contracts with ABI version < 2.4. Contract initial public key should be
    /// explicitly provided inside `initial_data` since ABI 2.4
    pub initial_pubkey: Option<String>,
}

impl DeploySet {
    pub fn some_with_tvc(tvc: Option<String>) -> Option<Self> {
        Some(Self {
            tvc,
            code: None,
            state_init: None,
            workchain_id: None,
            initial_data: None,
            initial_pubkey: None,
        })
    }

    pub fn get_state_init(&self, context: &ClientContext) -> ClientResult<Cell> {
        let error = |err| Err(Error::encode_deploy_message_failed(err));
        match (&self.tvc, &self.code, &self.state_init) {
            (Some(tvc_or_state_init), None, None) => {
                resolve_state_init_cell(context, tvc_or_state_init)
            }
            (None, Some(code), None) => {
                state_init_with_code(deserialize_cell_from_boc(context, code, "code")?.1)
            }
            (None, None, Some(state_init)) => {
                if self.initial_data.is_some() || self.initial_pubkey.is_some() {
                    error("Only `workchain_id` parameter is allowed if `state_init` parameter is provided")
                } else {
                    Ok(deserialize_cell_from_boc(context, state_init, "state init")?.1)
                }
            }
            (None, None, None) => error(
                "at least one of the `tvc`, `code` or `state_init` value should be specified.",
            ),
            _ => error("Only one of the `tvc`, `code` or `state_init` value should be specified."),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct CallSet {
    /// Function name that is being called.
    /// Or function id encoded as string in hex (starting with 0x).
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

        let func = match decode_abi_number::<u32>(&self.function_name) {
            Ok(id) => {
                &contract
                    .function_by_id(id, true)
                    .map_err(|e| Error::invalid_function_id(&self.function_name, e))?
                    .name
            }
            Err(_) => &self.function_name,
        }
        .clone();

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

    /// Signature ID to be used in data to sign preparing when CapSignatureWithId
    /// capability is enabled
    pub signature_id: Option<i32>,
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
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
        Err(Error::encode_deploy_message_failed(
            "Public key doesn't provided.",
        ))
    }
}

fn encode_deploy(
    context: Arc<ClientContext>,
    abi: &str,
    image: ContractImage,
    workchain: i32,
    call_set: &CallSet,
    pubkey: Option<&str>,
    signer: &Signer,
    processing_try_index: Option<u8>,
) -> ClientResult<(Vec<u8>, Option<Vec<u8>>, MsgAddressInt)> {
    let address = image.msg_address(workchain);
    Ok(match signer {
        Signer::None => {
            let message = ton_sdk::Contract::construct_deploy_message_json(
                &call_set.to_function_call_set(
                    pubkey,
                    processing_try_index,
                    &context,
                    abi,
                    false,
                )?,
                image,
                None,
                workchain,
            )
            .map_err(|err| Error::encode_run_message_failed(err, Some(&call_set.function_name)))?;
            (message.serialized_message, None, address)
        }
        _ => {
            let unsigned = ton_sdk::Contract::get_deploy_message_bytes_for_signing(
                &call_set.to_function_call_set(
                    pubkey,
                    processing_try_index,
                    &context,
                    &abi,
                    false,
                )?,
                image,
                workchain,
            )
            .map_err(|err| Error::encode_deploy_message_failed(err))?;
            (unsigned.message, Some(unsigned.data_to_sign), address)
        }
    })
}

fn encode_int_deploy(
    src: Option<MsgAddressInt>,
    context: Arc<ClientContext>,
    abi: &str,
    image: ContractImage,
    workchain_id: i32,
    call_set: &CallSet,
    ihr_disabled: bool,
    bounce: bool,
    value: CurrencyCollection,
) -> ClientResult<(Vec<u8>, MsgAddressInt)> {
    let address = image.msg_address(workchain_id);
    let message = ton_sdk::Contract::get_int_deploy_message_bytes(
        src,
        &call_set.to_function_call_set(None, None, &context, &abi, true)?,
        image,
        workchain_id,
        ihr_disabled,
        bounce,
        value,
    )
    .map_err(|err| Error::encode_deploy_message_failed(err))?;

    Ok((message, address))
}

fn encode_empty_deploy(
    image: ContractImage,
    workchain: i32,
) -> ClientResult<(Vec<u8>, Option<Vec<u8>>, MsgAddressInt)> {
    let address = image.msg_address(workchain);
    let message = ton_sdk::Contract::construct_deploy_message_no_constructor(image, workchain)
        .map_err(|x| Error::encode_deploy_message_failed(x))?;

    Ok((
        ton_sdk::Contract::serialize_message(&message)
            .map_err(|x| Error::encode_deploy_message_failed(x))?
            .0,
        None,
        address,
    ))
}

fn encode_empty_int_deploy(
    src: Option<MsgAddressInt>,
    image: ContractImage,
    workchain_id: i32,
    ihr_disabled: bool,
    bounce: bool,
    value: CurrencyCollection,
) -> ClientResult<(Vec<u8>, MsgAddressInt)> {
    let address = image.msg_address(workchain_id);
    let message = ton_sdk::Contract::construct_int_deploy_message_no_constructor(
        src,
        image,
        workchain_id,
        ihr_disabled,
        bounce,
        value,
    )
    .map_err(|x| Error::encode_deploy_message_failed(x))?;

    Ok((
        ton_sdk::Contract::serialize_message(&message)
            .map_err(|x| Error::encode_deploy_message_failed(x))?
            .0,
        address,
    ))
}

fn encode_run(
    context: Arc<ClientContext>,
    params: &ParamsOfEncodeMessage,
    abi: &str,
    call_set: &CallSet,
    pubkey: Option<&str>,
    processing_try_index: Option<u8>,
) -> ClientResult<(Vec<u8>, Option<Vec<u8>>, MsgAddressInt)> {
    let address = params
        .address
        .as_ref()
        .ok_or(Error::required_address_missing_for_encode_message())?;
    let address = account_decode(address)?;
    Ok(match params.signer {
        Signer::None => {
            let message = ton_sdk::Contract::construct_call_ext_in_message_json(
                address.clone(),
                &call_set.to_function_call_set(
                    pubkey,
                    processing_try_index,
                    &context,
                    abi,
                    false,
                )?,
                None,
            )
            .map_err(|err| Error::encode_run_message_failed(err, Some(&call_set.function_name)))?;
            (message.serialized_message, None, address)
        }
        _ => {
            let unsigned = ton_sdk::Contract::get_call_message_bytes_for_signing(
                address.clone(),
                &call_set.to_function_call_set(
                    pubkey,
                    processing_try_index,
                    &context,
                    abi,
                    false,
                )?,
            )
            .map_err(|err| Error::encode_run_message_failed(err, Some(&call_set.function_name)))?;

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
    context: Arc<ClientContext>,
    params: ParamsOfEncodeMessage,
) -> ClientResult<ResultOfEncodeMessage> {
    let abi_contract = params.abi.abi()?;
    let abi_string = params.abi.json_string()?;

    let public = params.signer.resolve_public_key(context.clone()).await?;
    let (message, data_to_sign, address) = if let Some(deploy_set) = params.deploy_set {
        let workchain = deploy_set
            .workchain_id
            .unwrap_or(context.config.abi.workchain);
        let mut image = create_tvc_image(
            &abi_string,
            abi_contract.data_map_supported(),
            deploy_set.initial_data.as_ref(),
            deploy_set.get_state_init(&context)?,
        )?;

        if abi_contract.data_map_supported() {
            required_public_key(update_pubkey(&deploy_set, &mut image, &public)?)?;
        } else if deploy_set.initial_pubkey.is_some() {
            return Err(Error::initial_pubkey_not_supported(abi_contract.version()));
        }

        if let Some(call_set) = &params.call_set {
            encode_deploy(
                context.clone(),
                &abi_string,
                image,
                workchain,
                call_set,
                public.as_ref().map(|x| x.as_str()),
                &params.signer,
                params.processing_try_index,
            )?
        } else {
            encode_empty_deploy(image, workchain)?
        }
    } else if let Some(call_set) = &params.call_set {
        encode_run(
            context.clone(),
            &params,
            &abi_string,
            call_set,
            public.as_ref().map(|x| x.as_str()),
            params.processing_try_index,
        )?
    } else {
        return Err(Error::missing_required_call_set_for_encode_message());
    };

    let data_to_sign = extend_data_to_sign(&context, params.signature_id, data_to_sign).await?;
    let (message, data_to_sign) =
        try_to_sign_message(context.clone(), &abi_string, message, data_to_sign, &params.signer).await?;

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
    /// Contract ABI. Can be None if both deploy_set and call_set are None.
    pub abi: Option<Abi>,

    /// Target address the message will be sent to.
    ///
    /// Must be specified in case of non-deploy message.
    pub address: Option<String>,

    /// Source address of the message.
    pub src_address: Option<String>,

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

    /// Value in nanotokens to be sent with message.
    pub value: String,

    /// Flag of bounceable message. Default is true.
    pub bounce: Option<bool>,

    /// Enable Instant Hypercube Routing for the message. Default is false.
    pub enable_ihr: Option<bool>,
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug)]
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
pub fn encode_internal_message(
    context: Arc<ClientContext>,
    params: ParamsOfEncodeInternalMessage,
) -> ClientResult<ResultOfEncodeInternalMessage> {
    let src_address = match params.src_address {
        Some(ref addr) => Some(account_decode(addr)?),
        None => None,
    };
    let ihr_disabled = !params.enable_ihr.unwrap_or(false);
    let bounce = params.bounce.unwrap_or(true);
    let value = CurrencyCollection::with_grams(u64::from_str(&params.value).map_err(|err| {
        Error::encode_run_message_failed(
            err,
            params
                .call_set
                .as_ref()
                .map(|call_set| call_set.function_name.as_str()),
        )
    })?);

    let (message, address) = if let Some(deploy_set) = params.deploy_set {
        let abi = params
            .abi
            .ok_or_else(|| Error::invalid_abi("abi is undefined"))?;

        let abi_contract = abi.abi()?;
        let abi_string = abi.json_string()?;

        let workchain_id = deploy_set
            .workchain_id
            .unwrap_or(context.config.abi.workchain);

        let mut image = create_tvc_image(
            &abi_string,
            abi_contract.data_map_supported(),
            deploy_set.initial_data.as_ref(),
            deploy_set.get_state_init(&context)?,
        )?;
        if abi_contract.data_map_supported() {
            update_pubkey(&deploy_set, &mut image, &None)?;
        } else if deploy_set.initial_pubkey.is_some() {
            return Err(Error::initial_pubkey_not_supported(abi_contract.version()));
        }

        if let Some(call_set) = &params.call_set {
            encode_int_deploy(
                src_address,
                Arc::clone(&context),
                &abi_string,
                image,
                workchain_id,
                call_set,
                ihr_disabled,
                bounce,
                value,
            )?
        } else {
            encode_empty_int_deploy(
                src_address,
                image,
                workchain_id,
                ihr_disabled,
                bounce,
                value,
            )?
        }
    } else {
        let address = params
            .address
            .as_ref()
            .ok_or(Error::required_address_missing_for_encode_message())?;
        let address = account_decode(address)?;
        if let Some(call_set) = &params.call_set {
            let abi = params
                .abi
                .ok_or_else(|| Error::invalid_abi("abi is undefined"))?
                .json_string()?;
            let message = ton_sdk::Contract::construct_call_int_message_json(
                address.clone(),
                src_address,
                ihr_disabled,
                bounce,
                value,
                &call_set.to_function_call_set(None, None, &context, &abi, true)?,
            )
            .map_err(|err| Error::encode_run_message_failed(err, Some(&call_set.function_name)))?;

            (message.serialized_message, address)
        } else {
            let message = ton_sdk::Contract::construct_int_message_with_body(
                address.clone(),
                src_address,
                ihr_disabled,
                bounce,
                value,
                None,
            )
            .map_err(|err| Error::encode_run_message_failed(err, None))?;
            (message.serialized_message, address)
        }
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

    /// Destination address of the message
    ///
    /// Since ABI version 2.3 destination address of external inbound message is used in message
    /// body signature calculation. Should be provided when signed external inbound message body is
    /// created. Otherwise can be omitted.
    pub address: Option<String>,

    /// Signature ID to be used in data to sign preparing when CapSignatureWithId
    /// capability is enabled
    pub signature_id: Option<i32>,
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
    context: Arc<ClientContext>,
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
            let body = ever_abi::encode_function_call(
                &abi,
                &func,
                call.header.as_deref(),
                &call.input,
                params.is_internal,
                None,
                params.address.as_deref(),
            )
            .map_err(|err| Error::encode_run_message_failed(err, Some(&func)))?;
            (body, None)
        }
        _ => if params.is_internal {
            ever_abi::encode_function_call(
                &abi,
                &func,
                None,
                &call.input,
                true,
                None,
                params.address.as_deref(),
            )
            .map(|body| (body, None))
        } else {
            ever_abi::prepare_function_call_for_sign(
                &abi,
                &func,
                call.header.as_deref(),
                &call.input,
                params.address.as_deref(),
            )
            .map(|(body, data_to_sign)| (body, Some(data_to_sign)))
        }
        .map_err(|err| Error::encode_run_message_failed(err, Some(&func)))?,
    };
    let body: Vec<u8> = ever_block::boc::write_boc(
        &body
            .clone()
            .into_cell()
            .map_err(|err| Error::encode_run_message_failed(err, Some(&func)))?,
    )
    .map_err(|err| Error::encode_run_message_failed(err, Some(&func)))?;
    let data_to_sign = extend_data_to_sign(&context, params.signature_id, data_to_sign).await?;
    if let Some(unsigned) = &data_to_sign {
        if let Some(signature) = params.signer.sign(context.clone(), unsigned).await? {
            let pubkey = public.map(|string| hex_decode(&string)).transpose()?;
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
pub fn attach_signature(
    context: Arc<ClientContext>,
    params: ParamsOfAttachSignature,
) -> ClientResult<ResultOfAttachSignature> {
    let (boc, _) = deserialize_cell_from_boc(&context, &params.message, "message")?;
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
pub fn attach_signature_to_message_body(
    context: Arc<ClientContext>,
    params: ParamsOfAttachSignatureToMessageBody,
) -> ClientResult<ResultOfAttachSignatureToMessageBody> {
    let (boc, _) = deserialize_cell_from_boc(&context, &params.message, "message body")?;
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
