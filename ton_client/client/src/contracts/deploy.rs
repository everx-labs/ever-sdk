/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crypto::keys::{KeyPair, decode_public_key, account_encode};
use ton_sdk::{Contract, ContractImage};

use contracts::EncodedUnsignedMessage;

#[cfg(feature = "node_interaction")]
use futures::Stream;
#[cfg(feature = "node_interaction")]
use ton_sdk::Transaction;

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfDeploy {
    pub abi: serde_json::Value,
    pub constructorParams: serde_json::Value,
    pub initParams: Option<serde_json::Value>,
    pub imageBase64: String,
    pub keyPair: KeyPair,
    #[serde(default)]
    pub workchainId: i32,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfEncodeUnsignedDeployMessage {
    pub abi: serde_json::Value,
    pub constructorParams: serde_json::Value,
    pub initParams: Option<serde_json::Value>,
    pub imageBase64: String,
    pub publicKeyHex: String,
    #[serde(default)]
    pub workchainId: i32,
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
    pub initParams: Option<serde_json::Value>,
    pub imageBase64: String,
    pub keyPair: KeyPair,
    #[serde(default)]
    pub workchainId: i32,
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

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfGetDeployData {
    pub abi: Option<serde_json::Value>,
    pub initParams: Option<serde_json::Value>,
    pub imageBase64: Option<String>,
    pub publicKeyHex: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ResultOfGetDeployData {
    pub imageBase64: Option<String>,
    pub accountId: Option<String>,
    pub dataBase64: String,
}

#[cfg(feature = "node_interaction")]
pub(crate) fn deploy(_context: &mut ClientContext, params: ParamsOfDeploy) -> ApiResult<ResultOfDeploy> {
    debug!("-> contracts.deploy({})", params.constructorParams.to_string());

    let key_pair = params.keyPair.decode()?;

    let contract_image = create_image(&params.abi, params.initParams.as_ref(), &params.imageBase64, &key_pair.public)?;
    let account_id = contract_image.msg_address(params.workchainId);
    debug!("-> -> image prepared with address: {}", account_id);

    debug!("-> -> deploy");
    let tr = deploy_contract(&params, contract_image, &key_pair)?;
    debug!("-> -> deploy transaction: {}", tr. id());

    debug!("<-");
    super::run::check_transaction_status(&tr)?;
    Ok(ResultOfDeploy { address: account_encode(&account_id) })
}

pub(crate) fn get_address(_context: &mut ClientContext, params: ParamsOfGetDeployAddress) -> ApiResult<String> {
    let key_pair = params.keyPair.decode()?;
    let contract_image = create_image(&params.abi, params.initParams.as_ref(), &params.imageBase64, &key_pair.public)?;
    let account_id = contract_image.msg_address(params.workchainId);
    Ok(account_encode(&account_id))
}

pub(crate) fn encode_message(_context: &mut ClientContext, params: ParamsOfDeploy) -> ApiResult<ResultOfEncodeDeployMessage> {
    debug!("-> contracts.deploy.message({})", params.constructorParams.to_string());

    let keys = params.keyPair.decode()?;

    let contract_image = create_image(&params.abi, params.initParams.as_ref(), &params.imageBase64, &keys.public)?;
    let account_id = contract_image.msg_address(params.workchainId);
    debug!("image prepared with address: {}", account_encode(&account_id));
    let (message_body, message_id) = Contract::construct_deploy_message_json(
        "constructor".to_owned(),
        params.constructorParams.to_string().to_owned(),
        params.abi.to_string().to_owned(),
        contract_image,
        Some(&keys), params.workchainId).map_err(|err| ApiError::contracts_create_deploy_message_failed(err))?;

    debug!("<-");
    Ok(ResultOfEncodeDeployMessage {
        address: account_encode(&account_id),
        messageId: message_id.to_string(),
        messageIdBase64: message_id.to_base64().map_err(|err| ApiError::contracts_create_deploy_message_failed(err))?,
        messageBodyBase64: base64::encode(&message_body),
    })
}

pub(crate) fn get_deploy_data(_context: &mut ClientContext, params: ParamsOfGetDeployData) -> ApiResult<ResultOfGetDeployData> {
    debug!("-> contracts.run.message({}, {}, {})",
        &params.abi.clone().unwrap_or_default(),
        &params.imageBase64.clone().unwrap_or_default(),
        &params.initParams.clone().unwrap_or_default(),
    );


    let public = decode_public_key(&params.publicKeyHex)?;

    // if image provided use it to modify initial data
    let mut image = if let Some(image) = &params.imageBase64 {
        let bytes = base64::decode(&image)
            .map_err(|err| ApiError::contracts_invalid_image(err))?;
        let image = ContractImage::from_state_init_and_key(&mut bytes.as_slice(), &public)
            .map_err(|err| ApiError::contracts_image_creation_failed(err))?;

        image
    } else { // or create temporary one
        let mut image = ContractImage::new()
            .map_err(|err| ApiError::contracts_image_creation_failed(err))?;
        image.set_public_key(&public)
            .map_err(|err| ApiError::contracts_image_creation_failed(err))?;

        image
    };

    // if initial data provided add it to image
    if let Some(init_params) = params.initParams {
        let abi = params.abi.ok_or(ApiError::contracts_image_creation_failed("No ABI provided"))?;
        image.update_data(&init_params.to_string(), &abi.to_string())
            .map_err(|err| ApiError::contracts_image_creation_failed(err))?;
    }

    // data is always returned
    let data_base64 = base64::encode(&image.get_serialized_data()
        .map_err(|err| ApiError::contracts_image_creation_failed(err))?);

    // image is returned only if original image was provided
    // accountId is computed from image so it is returned only with image
    let (image_base64, account_id) = match params.imageBase64 {
        Some(_) => (
            Some(base64::encode(&image.serialize()
                .map_err(|err| ApiError::contracts_image_creation_failed(err))?)),
            Some(image.account_id().to_hex_string())
        ),
        None => (None, None),
    };

    debug!("<-");
    Ok(ResultOfGetDeployData {
        imageBase64: image_base64,
        accountId: account_id,
        dataBase64: data_base64
    })
}

pub(crate) fn encode_unsigned_message(_context: &mut ClientContext, params: ParamsOfEncodeUnsignedDeployMessage) -> ApiResult<ResultOfEncodeUnsignedDeployMessage> {
    let public = decode_public_key(&params.publicKeyHex)?;
    let image = create_image(&params.abi, params.initParams.as_ref(), &params.imageBase64, &public)?;
    let address_hex = account_encode(&image.msg_address(params.workchainId));
    let encoded = ton_sdk::Contract::get_deploy_message_bytes_for_signing(
        "constructor".to_owned(),
        params.constructorParams.to_string().to_owned(),
        params.abi.to_string().to_owned(),
        image, params.workchainId
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

use ed25519_dalek::PublicKey;
use types::{ApiResult, ApiError};

use client::ClientContext;

#[cfg(feature = "node_interaction")]
use ed25519_dalek::Keypair;

#[cfg(feature = "node_interaction")]
use ton_block::TransactionProcessingStatus;

fn create_image(abi: &serde_json::Value, init_params: Option<&serde_json::Value>, image_base64: &String, public_key: &PublicKey) -> ApiResult<ContractImage> {
    let bytes = base64::decode(image_base64)
        .map_err(|err| ApiError::contracts_invalid_image(err))?;
    let mut image = ContractImage::from_state_init_and_key(&mut bytes.as_slice(), public_key)
        .map_err(|err| ApiError::contracts_image_creation_failed(err))?;

    if let Some(params) = init_params {
        image.update_data(&params.to_string(), &abi.to_string())
            .map_err(|err| ApiError::contracts_image_creation_failed(
                format!("Failed to set initial data: {}", err)))?;
    }

    Ok(image)
}

#[cfg(feature = "node_interaction")]
fn deploy_contract(params: &ParamsOfDeploy, image: ContractImage, keys: &Keypair) -> ApiResult<Transaction> {
    let changes_stream = Contract::deploy_json(
        "constructor".to_owned(),
        params.constructorParams.to_string().to_owned(),
        params.abi.to_string().to_owned(),
        image, Some(keys), params.workchainId)
        .expect("Error deploying contract");

    let mut tr = None;
    for transaction in changes_stream.wait() {
        if let Err(e) = transaction {
            panic!("error next state getting: {}", e);
        }
        if let Ok(transaction) = transaction {
            debug!("-> -> deploy: {:?}", transaction.status);
            if transaction.status == TransactionProcessingStatus::Preliminary ||
                transaction.status == TransactionProcessingStatus::Proposed ||
                transaction.status == TransactionProcessingStatus::Finalized
            {
                tr = Some(transaction);
                break;
            }
        }
    }
    tr.ok_or(ApiError::contracts_deploy_transaction_missing())
}
