/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::crypto::keys::{KeyPair, decode_public_key, account_encode};
use crate::contracts::{EncodedUnsignedMessage, EncodedMessage};
use crate::contracts::run::RunFees;
use ton_sdk::{Contract, ContractImage, FunctionCallSet};

#[cfg(feature = "node_interaction")]
use ton_sdk::{NodeClient, RecievedTransaction};
#[cfg(feature = "node_interaction")]
use crate::contracts::run::{resolve_msg_sdk_error, retry_call};

const DEFAULT_WORKCHAIN: i32 = 0;


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployFunctionCallSet {
    pub abi: serde_json::Value,
    pub constructor_header: Option<serde_json::Value>,
    pub constructor_params: serde_json::Value,
}

impl Into<FunctionCallSet> for DeployFunctionCallSet {
    fn into(self) -> FunctionCallSet {
        FunctionCallSet {
            func: "constructor".to_owned(),
            header: self.constructor_header.map(|value| value.to_string().to_owned()),
            input: self.constructor_params.to_string(),
            abi: self.abi.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfDeploy {
    #[serde(flatten)]
    pub call_set: DeployFunctionCallSet,
    pub init_params: Option<serde_json::Value>,
    pub image_base64: String,
    pub key_pair: KeyPair,
    pub workchain_id: Option<i32>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfEncodeUnsignedDeployMessage {
    #[serde(flatten)]
    pub call_set: DeployFunctionCallSet,
    pub init_params: Option<serde_json::Value>,
    pub image_base64: String,
    pub public_key_hex: String,
    pub workchain_id: Option<i32>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResultOfEncodeUnsignedDeployMessage {
    pub encoded: EncodedUnsignedMessage,
    pub address_hex: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfGetDeployAddress {
    pub abi: serde_json::Value,
    pub init_params: Option<serde_json::Value>,
    pub image_base64: String,
    pub key_pair: KeyPair,
    pub workchain_id: Option<i32>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResultOfDeploy {
    pub address: String,
    pub already_deployed: bool,
    pub fees: Option<RunFees>,
    pub transaction: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfGetDeployData {
    pub abi: Option<serde_json::Value>,
    pub init_params: Option<serde_json::Value>,
    pub image_base64: Option<String>,
    pub public_key_hex: String,
    pub workchain_id: Option<i32>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResultOfGetDeployData {
    pub image_base64: Option<String>,
    pub account_id: Option<String>,
    pub address: Option<String>,
    pub data_base64: String,
}

#[cfg(feature = "node_interaction")]
pub(crate) async fn deploy(context: &mut ClientContext, params: ParamsOfDeploy) -> ApiResult<ResultOfDeploy> {
    debug!("-> contracts.deploy({:?})", params.call_set.clone());

    let key_pair = params.key_pair.decode()?;

    let contract_image = create_image(&params.call_set.abi, params.init_params.as_ref(), &params.image_base64, &key_pair.public)?;
    let account_id = contract_image.msg_address(params.workchain_id.unwrap_or(DEFAULT_WORKCHAIN));
    debug!("-> -> image prepared with address: {}", account_id);

    if check_deployed(context, &account_id).await? {
        return Ok(ResultOfDeploy { 
            address: account_encode(&account_id),
            already_deployed: true,
            fees: None,
            transaction: serde_json::Value::Null
        })
    }

    let client = context.get_client()?;
    debug!("-> -> deploy");
    let tr = deploy_contract(client, params, contract_image, &key_pair).await?;
    debug!("-> -> deploy transaction: {}", tr.parsed.id());

    debug!("<-");
    super::run::check_transaction_status(&tr.parsed, true, &account_id)?;
    Ok(ResultOfDeploy {
        address: account_encode(&account_id),
        already_deployed: false,
        fees: Some(tr.parsed.calc_fees().into()),
        transaction: tr.value
    })
}

pub(crate) fn get_address(_context: &mut ClientContext, params: ParamsOfGetDeployAddress) -> ApiResult<String> {
    let key_pair = params.key_pair.decode()?;
    let contract_image = create_image(&params.abi, params.init_params.as_ref(), &params.image_base64, &key_pair.public)?;
    let account_id = contract_image.msg_address(params.workchain_id.unwrap_or(DEFAULT_WORKCHAIN));
    Ok(account_encode(&account_id))
}

pub(crate) fn encode_message(context: &mut ClientContext, params: ParamsOfDeploy) -> ApiResult<EncodedMessage> {
    debug!("-> contracts.deploy.message({:?})", params.call_set.clone());

    let keys = params.key_pair.decode()?;
    let workchain = params.workchain_id.unwrap_or(DEFAULT_WORKCHAIN);

    let contract_image = create_image(&params.call_set.abi, params.init_params.as_ref(), &params.image_base64, &keys.public)?;
    let account_id = contract_image.msg_address(workchain);
    debug!("image prepared with address: {}", account_encode(&account_id));
    let msg = Contract::construct_deploy_message_json(
        params.call_set.into(),
        contract_image,
        Some(&keys),
        workchain,
        Some(context.get_client()?.timeouts()),
        None
    ).map_err(|err| ApiError::contracts_create_deploy_message_failed(err))?;

    debug!("<-");
    Ok(EncodedMessage::from_sdk_msg(msg))
}

pub(crate) fn get_deploy_data(_context: &mut ClientContext, params: ParamsOfGetDeployData) -> ApiResult<ResultOfGetDeployData> {
    debug!("-> contracts.deploy.data({}, {}, {})",
        &params.abi.clone().unwrap_or_default(),
        &params.image_base64.clone().unwrap_or_default(),
        &params.init_params.clone().unwrap_or_default(),
    );


    let public = decode_public_key(&params.public_key_hex)?;

    // if image provided use it to modify initial data
    let mut image = if let Some(image) = &params.image_base64 {
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
    if let Some(init_params) = params.init_params {
        let abi = params.abi.ok_or(ApiError::contracts_image_creation_failed("No ABI provided"))?;
        image.update_data(&init_params.to_string(), &abi.to_string())
            .map_err(|err| ApiError::contracts_image_creation_failed(err))?;
    }

    // data is always returned
    let data_base64 = base64::encode(&image.get_serialized_data()
        .map_err(|err| ApiError::contracts_image_creation_failed(err))?);

    // image is returned only if original image was provided
    // account_id is computed from image so it is returned only with image
    let (image_base64, account_id, address) = match params.image_base64 {
        Some(_) => (
            Some(base64::encode(&image.serialize()
                .map_err(|err| ApiError::contracts_image_creation_failed(err))?)),
            Some(image.account_id().to_hex_string()),
            Some(image.msg_address(params.workchain_id.unwrap_or(DEFAULT_WORKCHAIN)).to_string())
        ),
        None => (None, None, None),
    };

    debug!("<-");
    Ok(ResultOfGetDeployData {
        image_base64: image_base64,
        account_id: account_id,
        address,
        data_base64: data_base64
    })
}

pub(crate) fn encode_unsigned_message(context: &mut ClientContext, params: ParamsOfEncodeUnsignedDeployMessage) -> ApiResult<ResultOfEncodeUnsignedDeployMessage> {
    let public = decode_public_key(&params.public_key_hex)?;
    let image = create_image(&params.call_set.abi, params.init_params.as_ref(), &params.image_base64, &public)?;
    let workchain = params.workchain_id.unwrap_or(DEFAULT_WORKCHAIN);
    let address_hex = account_encode(&image.msg_address(workchain));
    let encoded = ton_sdk::Contract::get_deploy_message_bytes_for_signing(
        params.call_set.into(),
        image,
        workchain,
        Some(context.get_client()?.timeouts()),
        None
    ).map_err(|err| ApiError::contracts_create_deploy_message_failed(err))?;
    Ok(ResultOfEncodeUnsignedDeployMessage {
        encoded: EncodedUnsignedMessage {
            unsigned_bytes_base64: base64::encode(&encoded.message),
            bytes_to_sign_base64: base64::encode(&encoded.data_to_sign),
            expire: encoded.expire
        },
        address_hex: address_hex
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
async fn deploy_contract(client: &NodeClient, params: ParamsOfDeploy, image: ContractImage, keys: &Keypair) -> ApiResult<RecievedTransaction> {
    retry_call(client.timeouts().message_retries_count, |try_index: u8| {
        let call_set = params.call_set.clone();
        let workchain = params.workchain_id.unwrap_or(DEFAULT_WORKCHAIN);
        let image = image.clone();
        async move {
            let msg = Contract::construct_deploy_message_json(
                call_set.into(),
                image,
                Some(keys),
                workchain,
                Some(client.timeouts()),
                Some(try_index))
                .map_err(|err| ApiError::contracts_create_run_message_failed(err))?;
    
            let result = Contract::process_message(client, &msg, true).await;
            
            match result {
                Err(err) => 
                    Err(resolve_msg_sdk_error(
                        client, err, &msg.serialized_message, ApiError::contracts_deploy_failed).await?),
                Ok(tr) => Ok(tr)
            }
        }
    }).await
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
