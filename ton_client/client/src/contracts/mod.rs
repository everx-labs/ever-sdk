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

use crate::types::{ApiResult, base64_decode, ApiError};
use ton_sdk::{AbiContract, ContractImage};
use ton_block::{CommonMsgInfo, Deserializable};
use ton_types::deserialize_tree_of_cells;
use std::io::Cursor;
use crate::crypto::keys::{
    account_decode,
    account_encode_ex,
    AccountAddressType,
    Base64AddressParams,
    decode_public_key
};

#[cfg(feature = "node_interaction")]
use ton_sdk::Contract;

pub(crate) mod deploy;
pub(crate) mod run;

#[cfg(feature = "node_interaction")]
pub(crate) mod load;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EncodedMessage {
    pub message_id: String,
    pub message_body_base64: String,
    pub expire: Option<u32>,
}

#[cfg(feature = "node_interaction")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfProcessMessage {
    pub abi: Option<serde_json::Value>,
    pub function_name: Option<String>,
    pub message: EncodedMessage,
    pub try_index: Option<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EncodedUnsignedMessage {
    pub unsigned_bytes_base64: String,
    pub bytes_to_sign_base64: String,
    pub expire: Option<u32>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfEncodeMessageWithSign {
    pub abi: serde_json::Value,
    pub unsigned_bytes_base64: String,
    pub sign_bytes_base64: String,
    pub public_key_hex: Option<String>,
    pub expire: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ParamsOfGetFunctionId {
    pub abi: serde_json::Value,
    pub function: String,
    pub input: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfConvertAddress {
    pub address: String,
    pub convert_to: AccountAddressType,
    pub base64_params: Option<Base64AddressParams>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfGetFunctionId {
    pub id: u32
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfGetCodeFromImage {
    pub image_base64: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResultOfGetCodeFromImage {
    pub code_base64: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfConvertAddress {
    pub address: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InputBoc {
    pub boc_base64: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfGetBocHash {
    pub hash: String,
}

use ton_sdk;
use crate::dispatch::DispatchTable;
use crate::client::ClientContext;

pub(crate) fn encode_message_with_sign(_context: &mut ClientContext, params: ParamsOfEncodeMessageWithSign) -> ApiResult<EncodedMessage> {
    let key_array: [u8; ed25519_dalek::PUBLIC_KEY_LENGTH];
    let public_key = if let Some(key) = params.public_key_hex {
        key_array = decode_public_key(&key)?.to_bytes();
        Some(key_array.as_ref())
    } else {
        None
    };
    let (body, id) = ton_sdk::Contract::add_sign_to_message(
        params.abi.to_string(),
        &base64_decode(&params.sign_bytes_base64)?,
        public_key,
        &base64_decode(&params.unsigned_bytes_base64)?
    ).map_err(|err|ApiError::contracts_encode_message_with_sign_failed(err))?;
    Ok(EncodedMessage {
        message_id: id.to_string(),
        message_body_base64: base64::encode(&body),
        expire: params.expire
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

    let bytes = base64::decode(&params.image_base64)
        .map_err(|err| ApiError::contracts_invalid_image(err))?;
    let mut reader = Cursor::new(bytes);
    let image = ContractImage::from_state_init(&mut reader)
        .map_err(|err| ApiError::contracts_image_creation_failed(err))?;

    debug!("<-");
    Ok(ResultOfGetCodeFromImage {
        code_base64: base64::encode(&image.get_serialized_code()
            .map_err(|err| ApiError::contracts_image_creation_failed(err))?),
    })
}

pub(crate) fn convert_address(_context: &mut ClientContext, params: ParamsOfConvertAddress) -> ApiResult<ResultOfConvertAddress> {
    debug!("-> contracts.image.code({}, {:?}, {:?})", params.address, params.convert_to, params.base64_params);
    let address = account_decode(&params.address)?;
    Ok(ResultOfConvertAddress {
        address: account_encode_ex(&address, params.convert_to, params.base64_params)?,
    })
}

fn decode_boc_base64(boc_base64: &String) -> ApiResult<ton_types::Cell> {
    let bytes = base64_decode(boc_base64)?;
    deserialize_tree_of_cells(&mut bytes.as_slice())
        .map_err(|err| ApiError::contracts_invalid_boc(err))
}

pub(crate) fn get_boc_root_hash(_context: &mut ClientContext, params: InputBoc) -> ApiResult<ResultOfGetBocHash> {
    debug!("-> contracts.boc.hash({})", params.boc_base64);
    let cells = decode_boc_base64(&params.boc_base64)?;
    Ok(ResultOfGetBocHash {
        hash: format!("{:x}", cells.repr_hash()),
    })
}

#[cfg(feature = "node_interaction")]
pub(crate) async fn send_message(context: &mut ClientContext, params: EncodedMessage) -> ApiResult<()> {
    debug!("-> contracts.send.message({}, {})", params.message_id, params.expire.unwrap_or_default());
    
    let msg = base64_decode(&params.message_body_base64)?;
    let id = crate::types::hex_decode(&params.message_id)?;
    let client = context.get_client()?;
    client.send_message(&id, &msg)
        .await
        .map_err(|err| ApiError::contracts_send_message_failed(err))
}

#[cfg(feature = "node_interaction")]
pub(crate) async fn process_message(context: &mut ClientContext, params: ParamsOfProcessMessage) -> ApiResult<run::ResultOfRun> {
    debug!("-> contracts.process.message({}, {})", 
        params.message.message_id,
        params.message.expire.unwrap_or_default());
    
    let msg = base64_decode(&params.message.message_body_base64)?;
    let client = context.get_client()?;
    let transaction = Contract::process_serialized_message(
        client,
        &params.message.message_id.into(),
        &msg,
        params.message.expire,
        params.try_index.unwrap_or(0))
        .await
        .map_err(|err| crate::types::apierror_from_sdkerror(err, ApiError::contracts_process_message_failed))?;

    run::process_transaction(client, transaction, params.abi, params.function_name).await
}

pub(crate) fn parse_message(_context: &mut ClientContext, params: InputBoc) -> ApiResult<serde_json::Value> {
    debug!("-> contracts.boc.hash({})", params.boc_base64);
    let cells = decode_boc_base64(&params.boc_base64)?;
    let mut message = ton_block::Message::default();
    message.read_from(&mut cells.into())
        .map_err(|err| ApiError::contracts_invalid_boc(err))?;
    // TODO: serialize via ton-block-json when it is opened
    let address = match message.header() {
        CommonMsgInfo::IntMsgInfo(ref header) => header.dst.to_string(),
        CommonMsgInfo::ExtInMsgInfo(ref header) => header.dst.to_string(),
        CommonMsgInfo::ExtOutMsgInfo(ref header) => header.dst.to_string()
    };
    Ok(json!({
        "dst": address
    }))
}

pub(crate) fn register(handlers: &mut DispatchTable) {
    // Load
    #[cfg(feature = "node_interaction")]
    handlers.spawn("contracts.load",
        |context: &mut crate::client::ClientContext, params: load::LoadParams| {
            let mut runtime = context.take_runtime()?;
            let result = runtime.block_on(load::load(context, params));
            context.runtime = Some(runtime);
            result
        });

    // Deploy
    #[cfg(feature = "node_interaction")]
    handlers.spawn("contracts.deploy",
        |context: &mut crate::client::ClientContext, params: deploy::ParamsOfDeploy| {
            let mut runtime = context.take_runtime()?;
            let result = runtime.block_on(deploy::deploy(context, params));
            context.runtime = Some(runtime);
            result
        });

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
        |context: &mut crate::client::ClientContext, params: run::ParamsOfRun| {
            let mut runtime = context.take_runtime()?;
            let result = runtime.block_on(run::run(context, params));
            context.runtime = Some(runtime);
            result
        });

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
        |context, params| run::local_run(context, params, true));
    handlers.spawn("contracts.run.local.msg",
        |context, params| run::local_run_msg(context, params, true));
    handlers.spawn("contracts.run.local.get",
        |context, params| run::local_run_get(context, params));
    handlers.spawn("contracts.run.fee",
        |context, params| run::local_run(context, params, false));
    handlers.spawn("contracts.run.fee.msg",
        |context, params| run::local_run_msg(context, params, false));

    // Contracts
    handlers.spawn("contracts.encode_message_with_sign",
        encode_message_with_sign);
    handlers.spawn("contracts.function.id",
        get_function_id);
    handlers.spawn("contracts.image.code",
        get_code_from_image);

    // Addresses
    handlers.spawn("contracts.address.convert",
        convert_address);

    // Bag of cells
    handlers.spawn("contracts.boc.hash",
        get_boc_root_hash);
    handlers.spawn("contracts.parse.message",
        parse_message);

    // messages processing
    #[cfg(feature = "node_interaction")]
    handlers.spawn("contracts.send.message",
        |context: &mut crate::client::ClientContext, params: EncodedMessage| {
            let mut runtime = context.take_runtime()?;
            let result = runtime.block_on(send_message(context, params));
            context.runtime = Some(runtime);
            result
        });
    #[cfg(feature = "node_interaction")]
    handlers.spawn("contracts.process.message",
        |context: &mut crate::client::ClientContext, params: ParamsOfProcessMessage| {
            let mut runtime = context.take_runtime()?;
            let result = runtime.block_on(process_message(context, params));
            context.runtime = Some(runtime);
            result
        });
}
