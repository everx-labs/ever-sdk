use types::{ApiResult, hex_decode, base64_decode, ApiError};
use ton_sdk::{AbiContract, ContractImage};
use std::io::Cursor;

pub(crate) mod types;
pub(crate) mod deploy;
pub(crate) mod run;

#[cfg(feature = "node_interaction")]
pub(crate) mod load;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct EncodedMessage {
    pub messageId: String,
    pub messageIdBase64: String,
    pub messageBodyBase64: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct EncodedUnsignedMessage {
    pub unsignedBytesBase64: String,
    pub bytesToSignBase64: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfEncodeMessageWithSign {
    pub unsignedBytesBase64: String,
    pub signBytesBase64: String,
    pub publicKeyHex: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfGetFunctionId {
    pub abi: serde_json::Value,
    pub function: String,
    pub input: bool,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ResultOfGetFunctionId {
    pub id: u32
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfGetCodeFromImage {
    pub imageBase64: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfGetCodeFromImage {
    pub codeBase64: String,
}

use ton_sdk;
use tvm::types::UInt256;
use dispatch::DispatchTable;
use client::ClientContext;

pub(crate) fn encode_message_with_sign(_context: &mut ClientContext, params: ParamsOfEncodeMessageWithSign) -> ApiResult<EncodedMessage> {
    let (body, id) = ton_sdk::Contract::add_sign_to_message(
        &base64_decode(&params.signBytesBase64)?,
        &hex_decode(&params.publicKeyHex)?,
        &base64_decode(&params.unsignedBytesBase64)?
    ).map_err(|err|ApiError::contracts_encode_message_with_sign_failed(err))?;
    let id: UInt256 = id.into();
    Ok(EncodedMessage {
        messageId: hex::encode(&id),
        messageIdBase64: base64::encode(id.as_slice()),
        messageBodyBase64: base64::encode(&body),
    })
}

pub(crate) fn get_function_id(_context: &mut ClientContext, params: ParamsOfGetFunctionId) -> ApiResult<ResultOfGetFunctionId> {
    let contract = AbiContract::load(params.abi.to_string().as_bytes())
        .map_err(|err|ApiError::contracts_get_function_id_failed(err))?;

    let function = contract.function(&params.function)
        .map_err(|err|ApiError::contracts_get_function_id_failed(err))?;

    Ok(ResultOfGetFunctionId {
       id: if params.input { function.get_input_id() } else { function.get_input_id() }
    })
}

pub(crate) fn get_code_from_image(_context: &mut ClientContext, params: ParamsOfGetCodeFromImage) -> ApiResult<ResultOfGetCodeFromImage> {
    debug!("-> contracts.image.code()");

    let bytes = base64::decode(&params.imageBase64)
        .map_err(|err| ApiError::contracts_invalid_image(err))?;
    let mut reader = Cursor::new(bytes);
    let image = ContractImage::from_state_init(&mut reader)
        .map_err(|err| ApiError::contracts_image_creation_failed(err))?;

    debug!("<-");
    Ok(ResultOfGetCodeFromImage {
        codeBase64: base64::encode(&image.get_serialized_code()
            .map_err(|err| ApiError::contracts_image_creation_failed(err))?),
    })
}

pub(crate) fn register(handlers: &mut DispatchTable) {
    // Load
    #[cfg(feature = "node_interaction")]
    handlers.spawn("contracts.load", load::load);

    // Deploy
    #[cfg(feature = "node_interaction")]
    handlers.spawn("contracts.deploy",
        deploy::deploy);

    handlers.spawn("contracts.deploy.message",
        deploy::encode_message);
    handlers.spawn("contracts.deploy.encode_unsigned_message",
        deploy::encode_unsigned_message);
    handlers.spawn("contracts.deploy.address",
        deploy::get_address);
    handlers.spawn("contracts.deploy.data",
        deploy::get_deploy_data);

    // Run
    #[cfg(feature = "node_interaction")]
    handlers.spawn("contracts.run",
        run::run);

    handlers.spawn("contracts.run.message",
        run::encode_message);
    handlers.spawn("contracts.run.encode_unsigned_message",
        run::encode_unsigned_message);
    handlers.spawn("contracts.run.output",
        run::decode_output);
    handlers.spawn("contracts.run.unknown.input",
        run::decode_unknown_input);
    handlers.spawn("contracts.run.unknown.output",
        run::decode_unknown_output);
    handlers.spawn("contracts.run.body",
        run::get_run_body);
    handlers.spawn("contracts.run.local",
        run::local_run);

    // Contracts
    handlers.spawn("contracts.encode_message_with_sign",
        encode_message_with_sign);
    handlers.spawn("contracts.function.id",
        get_function_id);
    handlers.spawn("contracts.image.code",
        get_code_from_image);
}
