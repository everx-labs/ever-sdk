/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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

use crate::crypto::keys::{KeyPair, decode_public_key, account_encode};
use crate::contracts::EncodedUnsignedMessage;
use crate::contracts::run::serialize_message;
use ton_sdk::{Contract, ContractImage, FunctionCallSet};


#[cfg(feature = "node_interaction")]
use ton_sdk::Transaction;
#[cfg(feature = "node_interaction")]
use ton_sdk::NodeClient;

const DEFAULT_WORKCHAIN: i32 = 0;


#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct DeployFunctionCallSet {
    pub abi: serde_json::Value,
    pub constructorHeader: Option<serde_json::Value>,
    pub constructorParams: serde_json::Value,
}

impl Into<FunctionCallSet> for DeployFunctionCallSet {
    fn into(self) -> FunctionCallSet {
        FunctionCallSet {
            func: "constructor".to_owned(),
            header: self.constructorHeader.map(|value| value.to_string().to_owned()),
            input: self.constructorParams.to_string(),
            abi: self.abi.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfDeploy {
    #[serde(flatten)]
    pub call_set: DeployFunctionCallSet,
    pub initParams: Option<serde_json::Value>,
    pub imageBase64: String,
    pub keyPair: KeyPair,
    pub workchainId: Option<i32>,
    pub try_index: Option<u8>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfEncodeUnsignedDeployMessage {
    #[serde(flatten)]
    pub call_set: DeployFunctionCallSet,
    pub initParams: Option<serde_json::Value>,
    pub imageBase64: String,
    pub publicKeyHex: String,
    pub workchainId: Option<i32>,
    pub try_index: Option<u8>,
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
    pub workchainId: Option<i32>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfDeploy {
    pub address: String,
    pub alreadyDeployed: bool,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfEncodeDeployMessage {
    pub address: String,
    pub messageId: String,
    pub messageBodyBase64: String,
    pub expire: Option<u32>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfGetDeployData {
    pub abi: Option<serde_json::Value>,
    pub initParams: Option<serde_json::Value>,
    pub imageBase64: Option<String>,
    pub publicKeyHex: String,
    pub workchainId: Option<i32>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ResultOfGetDeployData {
    pub imageBase64: Option<String>,
    pub accountId: Option<String>,
    pub address: Option<String>,
    pub dataBase64: String,
}

#[cfg(feature = "node_interaction")]
pub(crate) async fn deploy(context: &mut ClientContext, params: ParamsOfDeploy) -> ApiResult<ResultOfDeploy> {
    debug!("-> contracts.deploy({:?})", params.call_set.clone());

    let key_pair = params.keyPair.decode()?;

    let contract_image = create_image(&params.call_set.abi, params.initParams.as_ref(), &params.imageBase64, &key_pair.public)?;
    let account_id = contract_image.msg_address(params.workchainId.unwrap_or(DEFAULT_WORKCHAIN));
    debug!("-> -> image prepared with address: {}", account_id);

    if check_deployed(context, &account_id).await? {
        return Ok(ResultOfDeploy { 
            address: account_encode(&account_id),
            alreadyDeployed: true
        })
    }

    let client = context.get_client()?;
    debug!("-> -> deploy");
    let tr = deploy_contract(client, params, contract_image, &key_pair).await?;
    debug!("-> -> deploy transaction: {}", tr. id());

    debug!("<-");
    super::run::check_transaction_status(&tr)?;
    Ok(ResultOfDeploy {
        address: account_encode(&account_id),
        alreadyDeployed: false
    })
}

pub(crate) fn get_address(_context: &mut ClientContext, params: ParamsOfGetDeployAddress) -> ApiResult<String> {
    let key_pair = params.keyPair.decode()?;
    let contract_image = create_image(&params.abi, params.initParams.as_ref(), &params.imageBase64, &key_pair.public)?;
    let account_id = contract_image.msg_address(params.workchainId.unwrap_or(DEFAULT_WORKCHAIN));
    Ok(account_encode(&account_id))
}

pub(crate) fn encode_message(context: &mut ClientContext, params: ParamsOfDeploy) -> ApiResult<ResultOfEncodeDeployMessage> {
    debug!("-> contracts.deploy.message({:?})", params.call_set.clone());

    let keys = params.keyPair.decode()?;
    let workchain = params.workchainId.unwrap_or(DEFAULT_WORKCHAIN);

    let contract_image = create_image(&params.call_set.abi, params.initParams.as_ref(), &params.imageBase64, &keys.public)?;
    let account_id = contract_image.msg_address(workchain);
    debug!("image prepared with address: {}", account_encode(&account_id));
    let msg = Contract::construct_deploy_message_json(
        params.call_set.into(),
        contract_image,
        Some(&keys),
        workchain,
        Some(context.get_client()?.timeouts()),
        params.try_index
    ).map_err(|err| ApiError::contracts_create_deploy_message_failed(err))?;

    let (body, id) = serialize_message(msg.message)?;

    debug!("<-");
    Ok(ResultOfEncodeDeployMessage {
        address: account_encode(&account_id),
        messageId: id,
        messageBodyBase64: base64::encode(&body),
        expire: msg.expire
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
    let (image_base64, account_id, address) = match params.imageBase64 {
        Some(_) => (
            Some(base64::encode(&image.serialize()
                .map_err(|err| ApiError::contracts_image_creation_failed(err))?)),
            Some(image.account_id().to_hex_string()),
            Some(image.msg_address(params.workchainId.unwrap_or(DEFAULT_WORKCHAIN)).to_string())
        ),
        None => (None, None, None),
    };

    debug!("<-");
    Ok(ResultOfGetDeployData {
        imageBase64: image_base64,
        accountId: account_id,
        address,
        dataBase64: data_base64
    })
}

pub(crate) fn encode_unsigned_message(context: &mut ClientContext, params: ParamsOfEncodeUnsignedDeployMessage) -> ApiResult<ResultOfEncodeUnsignedDeployMessage> {
    let public = decode_public_key(&params.publicKeyHex)?;
    let image = create_image(&params.call_set.abi, params.initParams.as_ref(), &params.imageBase64, &public)?;
    let workchain = params.workchainId.unwrap_or(DEFAULT_WORKCHAIN);
    let address_hex = account_encode(&image.msg_address(workchain));
    let encoded = ton_sdk::Contract::get_deploy_message_bytes_for_signing(
        params.call_set.into(),
        image,
        workchain,
        Some(context.get_client()?.timeouts()),
        params.try_index
    ).map_err(|err| ApiError::contracts_create_deploy_message_failed(err))?;
    Ok(ResultOfEncodeUnsignedDeployMessage {
        encoded: EncodedUnsignedMessage {
            unsignedBytesBase64: base64::encode(&encoded.message),
            bytesToSignBase64: base64::encode(&encoded.data_to_sign),
            expire: encoded.expire
        },
        addressHex: address_hex
    })
}

// Internals

use ed25519_dalek::PublicKey;
use crate::types::{ApiResult, ApiError};

use crate::client::ClientContext;

#[cfg(feature = "node_interaction")]
use crate::queries::query::{query, ParamsOfQuery};
#[cfg(feature = "node_interaction")]
use ed25519_dalek::Keypair;
#[cfg(feature = "node_interaction")]
use ton_block::{AccountStatus};
#[cfg(feature = "node_interaction")]
use ton_sdk::json_helper::account_status_to_u8;

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
async fn deploy_contract(client: &NodeClient, params: ParamsOfDeploy, image: ContractImage, keys: &Keypair) -> ApiResult<Transaction> {
    Contract::deploy_json(
        client,
        params.call_set.into(),
        image,
        Some(keys),
        params.workchainId.unwrap_or(DEFAULT_WORKCHAIN))
            .await
            .map_err(|err| crate::types::apierror_from_sdkerror(err, ApiError::contracts_run_failed))
}

#[cfg(feature = "node_interaction")]
async fn check_deployed(context: &mut ClientContext, address: &ton_block::MsgAddressInt) -> ApiResult<bool> {
    let filter = json!({
        "id": { "eq": address.to_string() },
        "acc_type": { "eq":  account_status_to_u8(AccountStatus::AccStateActive) }
    }).to_string();

    let result = query(context, ParamsOfQuery {
        table: ton_sdk::types::CONTRACTS_TABLE_NAME.to_owned(),
        filter,
        result: "id".to_owned(),
        limit: None,
        order: None
    })
        .await?;

    Ok(result.result.as_array().and_then(|value| value.get(0)).is_some())
}
