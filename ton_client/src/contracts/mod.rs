use types::{ApiResult, hex_decode, base64_decode, ApiError};

pub(crate) mod types;
pub(crate) mod deploy;
pub(crate) mod run;

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

use ton_sdk;
use tvm::types::UInt256;
use dispatch::DispatchTable;
use client::ClientContext;
use contracts::deploy::ParamsOfDeploy;

pub(crate) fn encode_message_with_sign(context: &mut ClientContext, params: ParamsOfEncodeMessageWithSign) -> ApiResult<EncodedMessage> {
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

pub(crate) fn register(handlers: &mut DispatchTable) {
    // Load

    handlers.spawn("contracts.load", load::load);

    // Deploy

    handlers.spawn("contracts.deploy",
        deploy::deploy);
    handlers.spawn("contracts.deploy.message",
        deploy::encode_message);
    handlers.spawn("contracts.deploy.encode_unsigned_message",
        deploy::encode_unsigned_message);
    handlers.spawn("contracts.deploy.address",
        deploy::get_address);

    // Run

    handlers.spawn("contracts.run",
        run::run);
    handlers.spawn("contracts.run.message",
        run::encode_message);
    handlers.spawn("contracts.run.encode_unsigned_message",
        run::encode_unsigned_message);
    handlers.spawn("contracts.run.output",
        run::decode_output);

    // Contracts

    handlers.spawn("contracts.send.grams.message",
        deploy::encode_send_grams_message);
    handlers.spawn("contracts.encode_message_with_sign",
        encode_message_with_sign);
    handlers.spawn("contracts.run.local",
        run::local_run);
}
