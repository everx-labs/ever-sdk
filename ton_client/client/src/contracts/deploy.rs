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

use crate::crypto::keys::{KeyPair};
use crate::encoding::{account_encode};
use crate::crypto::internal::{decode_public_key};
use crate::contracts::{EncodedUnsignedMessage, EncodedMessage};
use crate::contracts::run::RunFees;
use ton_sdk::{Contract, ContractImage, FunctionCallSet};

#[cfg(feature = "node_interaction")]
use ton_sdk::{NodeClient, ReceivedTransaction};
#[cfg(feature = "node_interaction")]
use crate::contracts::run::{resolve_msg_sdk_error, retry_call};

const DEFAULT_WORKCHAIN: i32 = 0;


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployFunctionCallSet {
    /// contract ABI
    pub abi: serde_json::Value,
    /// message header according to contract ABI
    pub constructor_header: Option<serde_json::Value>,
    /// function input parameters according to contract ABI
    pub constructor_params: serde_json::Value,
}

impl DeployFunctionCallSet {
    pub fn function_name() -> &'static str {
        "constructor"
    }
}

impl Into<FunctionCallSet> for DeployFunctionCallSet {
    fn into(self) -> FunctionCallSet {
        FunctionCallSet {
            func: Self::function_name().to_owned(),
            header: self.constructor_header.map(|value| value.to_string().to_owned()),
            input: self.constructor_params.to_string(),
            abi: self.abi.to_string(),
        }
    }
}

#[doc(summary="Method that deploys a contract")]
/// Method creates a deploy message signed with key_pair, sends it to the target workchain,
/// waits for the result transaction and outbound messages and decodes the parameters
/// returned by the constructor, using ABI.
///
/// If the contract implements Pragma Expire, the method repeats the algorithm
/// for message_retries_count times
/// if the message was not delivered during message_expiration_timeout (see setup.SetupParams).
///
/// If the contract does not implement Pragma Expire - the method waits for the result
/// transaction for message_processing_timeout (see setup.SetupParams),
/// and exits with 1012 original error
/// Before exiting, message is processed on the local transaction executor to check the possible reason
/// why the transaction was not finalized (see resolve_error method documentation)
/// and  if such error is found returns it,
/// if not - returns disclaimer that the local execution was successful.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfDeploy {
    #[serde(flatten)]
    pub call_set: DeployFunctionCallSet,
    /// list of initial values for contract public variables
    pub init_params: Option<serde_json::Value>,
    /// tvc converted to base64
    pub image_base64: String,
    /// key pair for signature
    pub key_pair: KeyPair,
    /// target workchain for deploy
    pub workchain_id: Option<i32>,
    /// [1.0.0] will be deprecated
    pub try_index: Option<u8>,
}

#[doc(summary="Method that creates an unsigned deploy message")]
/// Method that creates an unsigned deploy message that can be signed,
/// for instance, outside the application
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfEncodeUnsignedDeployMessage {
    #[serde(flatten)]
    pub call_set: DeployFunctionCallSet,
    /// list of initial values of contract public variables.
    /// They are placed into the persistent storage of an account and influence the contract future address.
    pub init_params: Option<serde_json::Value>,
    /// initial contract image - tvc file - result of contract compilation - converted to base64
    pub image_base64: String,
    /// public key, that will be placed to the persistent storage along with init_params. It also influences the future address.
    pub public_key_hex: String,
    /// target workchain for deploy
    pub workchain_id: Option<i32>,
    /// [1.0.0] will be deprecated
    pub try_index: Option<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResultOfEncodeUnsignedDeployMessage {
    /// structure with encoded unsigned message
    pub encoded: EncodedUnsignedMessage,
    /// future contract address in raw format
    pub address_hex: String,
}

#[doc(summary="Method that calculates the future contract address")]
/// Method that calculates the future contract address
/// from contract image, initial parameters, key pair and target workchain
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfGetDeployAddress {
    /// contract ABI
    pub abi: serde_json::Value,
    /// list of initial values of contract public variables.
    /// They are placed into the persistent strorage of an account and influence the contract future address.
    pub init_params: Option<serde_json::Value>,
    /// initial contract image - tvc file - result of contract compilation - converted to base64
    pub image_base64: String,
    /// key pair for signature
    pub key_pair: KeyPair,
    /// target workchain for deploy
    pub workchain_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResultOfDeploy {
    /// account address
    pub address: String,
    /// flag that indicates that the contract was already deployed
    pub already_deployed: bool,
    /// fees collected for deploy
    pub fees: Option<RunFees>,
    /// transaction in json format with structure according to graphql schema
    pub transaction: serde_json::Value,
}

#[doc(summary="Calculates contract deploy image and/or initial data from initial image and/or data ")]
/// Places the initial data and public key into initial image and
/// generates the deploy image and deploy data of the contract.
/// If initial image is not provided - will calculate deploy data only
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfGetDeployData {
    /// contract ABI
    pub abi: Option<serde_json::Value>,
    /// list of initial values of contract public variables.
    /// They are placed into the persistent storage of an account and influence the contract future address.
    pub init_params: Option<serde_json::Value>,
    /// initial contract image - tvc file - result of contract compilation - converted to base64
    pub image_base64: Option<String>,
    /// public key, that will be placed to the persistent storage along with init_params. It also influences the future address.
    pub public_key_hex: String,
    /// target workchain for deploy
    pub workchain_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResultOfGetDeployData {
    /// Deploy contract image: initial contract data and public key are placed into the initial contract image. Returned only if initial image is specified.
    pub image_base64: Option<String>,
    /// Account identifier, calculated from the result contract image. Returned only if initial image is specified.
    pub account_id: Option<String>,
    /// Full contract address, including  account_id and workchain. Returned only if initial image is specified.
    pub address: Option<String>,
    /// The deploy data of the contract: initial parameters and public key to be placed into the persistent strorage -
    /// into <data> field of <StateInit> structure - see p. 4.1.6 сblockchain spec. Always returned.
    pub data_base64: String,
}

#[cfg(feature = "node_interaction")]
pub(crate) async fn deploy(context: &mut ClientContext, params: ParamsOfDeploy) -> ApiResult<ResultOfDeploy> {
    trace!("-> contracts.deploy({:?})", params.call_set.clone());

    let key_pair = params.key_pair.decode()?;

    let contract_image = create_image(&params.call_set.abi, params.init_params.as_ref(), &params.image_base64, &key_pair.public)?;
    let account_id = contract_image.msg_address(params.workchain_id.unwrap_or(DEFAULT_WORKCHAIN));
    trace!("-> -> image prepared with address: {}", account_id);

    if check_deployed(context, &account_id).await? {
        return Ok(ResultOfDeploy {
            address: account_encode(&account_id),
            already_deployed: true,
            fees: None,
            transaction: serde_json::Value::Null
        })
    }

    let client = context.get_client()?;
    trace!("-> -> deploy");
    let tr = deploy_contract(client, params, contract_image, &key_pair)
        .await
        .map_err(|err| err
            .add_function(Some(&DeployFunctionCallSet::function_name()))
            .add_network_url(client)
        )?;
    trace!("-> -> deploy transaction: {}", tr.parsed.id());

    trace!("<-");
    super::run::check_transaction_status(&tr.parsed, true, &account_id, None)
        .map_err(|err| err
            .add_function(Some(&DeployFunctionCallSet::function_name()))
            .add_network_url(client)
        )?;
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
    trace!("-> contracts.deploy.message({:?})", params.call_set.clone());

    let keys = params.key_pair.decode()?;
    let workchain = params.workchain_id.unwrap_or(DEFAULT_WORKCHAIN);

    let contract_image = create_image(&params.call_set.abi, params.init_params.as_ref(), &params.image_base64, &keys.public)?;
    let account_id = contract_image.msg_address(workchain);
    trace!("image prepared with address: {}", account_encode(&account_id));
    let msg = Contract::construct_deploy_message_json(
        params.call_set.into(),
        contract_image,
        Some(&keys),
        workchain,
        Some(context.get_client()?.timeouts()),
        params.try_index
    ).map_err(|err| ApiError::contracts_create_deploy_message_failed(err))?;

    trace!("<-");
    Ok(EncodedMessage::from_sdk_msg(msg))
}

pub(crate) fn get_deploy_data(_context: &mut ClientContext, params: ParamsOfGetDeployData) -> ApiResult<ResultOfGetDeployData> {
    trace!("-> contracts.deploy.data({}, {}, {})",
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

    trace!("<-");
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
        params.try_index
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
use crate::error::{ApiResult, ApiError};

use crate::client::ClientContext;

#[cfg(feature = "node_interaction")]
use crate::queries::{query_collection, ParamsOfQueryCollection};
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
async fn deploy_contract(client: &NodeClient, params: ParamsOfDeploy, image: ContractImage, keys: &Keypair) -> ApiResult<ReceivedTransaction> {
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
                .map_err(|err| ApiError::contracts_create_deploy_message_failed(err))?;

            let result = Contract::process_message(client, &msg, true).await;

            match result {
                Err(err) =>
                    Err(resolve_msg_sdk_error(
                        client, err, &msg, None,
                        ApiError::contracts_deploy_failed).await?
                    ),
                Ok(tr) => Ok(tr)
            }
        }
    }).await
}

#[cfg(feature = "node_interaction")]
async fn check_deployed(context: &mut ClientContext, address: &ton_block::MsgAddressInt) -> ApiResult<bool> {
    let filter = Some(json!({
        "id": { "eq": address.to_string() },
        "acc_type": { "eq":  account_status_to_u8(AccountStatus::AccStateActive) }
    }));

    let result = query_collection(context, ParamsOfQueryCollection {
        collection: ton_sdk::types::CONTRACTS_TABLE_NAME.to_owned(),
        filter,
        result: "id".to_owned(),
        limit: None,
        order: None,
    })
        .await?;

    Ok(result.result.get(0).is_some())
}
