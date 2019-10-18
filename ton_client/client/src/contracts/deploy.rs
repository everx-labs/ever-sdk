use crypto::keys::{KeyPair, decode_public_key, account_encode, generic_id_encode};
use ton_sdk::{Contract, ContractImage};

use contracts::EncodedUnsignedMessage;

#[cfg(feature = "node_interaction")]
use tvm::block::TransactionId;
#[cfg(feature = "node_interaction")]
use futures::Stream;
#[cfg(feature = "node_interaction")]
use crypto::keys::u256_encode;

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfDeploy {
    pub abi: serde_json::Value,
    pub constructorParams: serde_json::Value,
    pub imageBase64: String,
    pub keyPair: KeyPair,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfEncodeUnsignedDeployMessage {
    pub abi: serde_json::Value,
    pub constructorParams: serde_json::Value,
    pub imageBase64: String,
    pub publicKeyHex: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ResultOfEncodeUnsignedDeployMessage {
    pub encoded: EncodedUnsignedMessage,
    pub addressHex: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfGetDeployAddress {
    pub abi: serde_json::Value,
    pub imageBase64: String,
    pub keyPair: KeyPair,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfDeploy {
    pub address: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfEncodeDeployMessage {
    pub address: String,
    pub messageId: String,
    pub messageIdBase64: String,
    pub messageBodyBase64: String,
}

#[cfg(feature = "node_interaction")]
pub(crate) fn deploy(_context: &mut ClientContext, params: ParamsOfDeploy) -> ApiResult<ResultOfDeploy> {
    debug!("-> contracts.deploy({})", params.constructorParams.to_string());

    let key_pair = params.keyPair.decode()?;

    let contract_image = create_image(&params.imageBase64, &key_pair.public)?;
    let account_id = contract_image.account_id(0);
    debug!("-> -> image prepared with address: {}", account_id);

    debug!("-> -> deploy");
    let tr_id = deploy_contract(&params, contract_image, &key_pair)?;
    debug!("-> -> deploy transaction: {}", u256_encode(&tr_id.clone().into()));

    let tr_id_hex = tr_id.to_hex_string();

    debug!("load transaction {}", tr_id_hex);
    let tr = super::run::load_transaction(&tr_id);

    debug!("<-");
    if tr.tr().is_aborted() {
        debug!("Transaction aborted");
        super::run::get_result_from_block_transaction(tr.tr())?;
        Err(ApiError::contracts_deploy_transaction_aborted())
    } else {
        Ok(ResultOfDeploy {
            address: account_encode(&account_id)
        })
    }
}

pub(crate) fn get_address(_context: &mut ClientContext, params: ParamsOfGetDeployAddress) -> ApiResult<String> {
    let key_pair = params.keyPair.decode()?;
    let contract_image = create_image(&params.imageBase64, &key_pair.public)?;
    let account_id = contract_image.account_id(0);
    Ok(account_encode(&account_id))
}

pub(crate) fn encode_message(_context: &mut ClientContext, params: ParamsOfDeploy) -> ApiResult<ResultOfEncodeDeployMessage> {
    debug!("-> contracts.deploy.message({})", params.constructorParams.to_string());

    let keys = params.keyPair.decode()?;

    let contract_image = create_image(&params.imageBase64, &keys.public)?;
    let account_id = contract_image.account_id(0);
    debug!("image prepared with address: {}", account_encode(&account_id));
    let account_id = contract_image.account_id(0);
    let (message_body, message_id) = Contract::construct_deploy_message_json(
        "constructor".to_owned(),
        params.constructorParams.to_string().to_owned(),
        params.abi.to_string().to_owned(),
        contract_image,
        Some(&keys)).map_err(|err| ApiError::contracts_create_deploy_message_failed(err))?;

    debug!("<-");
    Ok(ResultOfEncodeDeployMessage {
        address: account_encode(&account_id),
        messageId: generic_id_encode(&message_id),
        messageIdBase64: base64::encode(message_id.data.as_slice()),
        messageBodyBase64: base64::encode(&message_body),
    })
}

pub(crate) fn encode_unsigned_message(_context: &mut ClientContext, params: ParamsOfEncodeUnsignedDeployMessage) -> ApiResult<ResultOfEncodeUnsignedDeployMessage> {
    let public = decode_public_key(&params.publicKeyHex)?;
    let image = create_image(&params.imageBase64, &public)?;
    let address_hex = account_encode(&image.account_id(0));
    let encoded = ton_sdk::Contract::get_deploy_message_bytes_for_signing(
        "constructor".to_owned(),
        params.constructorParams.to_string().to_owned(),
        params.abi.to_string().to_owned(),
        image
    ).map_err(|err| ApiError::contracts_create_deploy_message_failed(err))?;
    Ok(ResultOfEncodeUnsignedDeployMessage {
        encoded: EncodedUnsignedMessage {
            unsignedBytesBase64: base64::encode(&encoded.message),
            bytesToSignBase64: base64::encode(&encoded.data_to_sign),
        },
        addressHex: address_hex
    })
}

// Internals

use std::io::Cursor;
use ed25519_dalek::PublicKey;
use types::{ApiResult, ApiError};

use client::ClientContext;

#[cfg(feature = "node_interaction")]
use ed25519_dalek::Keypair;

#[cfg(feature = "node_interaction")]
use tvm::block::TransactionProcessingStatus;

fn create_image(image_base64: &String, public_key: &PublicKey) -> ApiResult<ContractImage> {
    let bytes = base64::decode(image_base64)
        .map_err(|err| ApiError::contracts_deploy_invalid_image(err))?;
    let mut reader = Cursor::new(bytes);
    ContractImage::from_state_init_and_key(&mut reader, public_key)
        .map_err(|err| ApiError::contracts_deploy_image_creation_failed(err))
}

#[cfg(feature = "node_interaction")]
fn deploy_contract(params: &ParamsOfDeploy, image: ContractImage, keys: &Keypair) -> ApiResult<TransactionId> {
    let changes_stream = Contract::deploy_json(
        "constructor".to_owned(),
        params.constructorParams.to_string().to_owned(),
        params.abi.to_string().to_owned(),
        image,
        Some(keys))
        .expect("Error deploying contract");

    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(state) = state {
            debug!("-> -> deploy: {:?}", state.status);
            if state.status == TransactionProcessingStatus::Preliminary ||
                state.status == TransactionProcessingStatus::Proposed ||
                state.status == TransactionProcessingStatus::Finalized
            {
                tr_id = Some(state.id.clone());
                break;
            }
        }
    }
    tr_id.ok_or(ApiError::contracts_deploy_transaction_missing())
}
