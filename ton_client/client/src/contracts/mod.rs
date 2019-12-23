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

use types::{ApiResult, hex_decode, base64_decode, ApiError};
use ton_sdk::{AbiContract, ContractImage};
use std::io::Cursor;
use crypto::keys::{account_decode, account_encode_ex, AccountAddressType, Base64AddressParams};

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
pub(crate) struct ParamsOfConvertAddress {
    pub address: String,
    pub convertTo: AccountAddressType,
    pub base64Params: Option<Base64AddressParams>,
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

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfConvertAddress {
    pub address: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfGetBocHash {
    pub bocBase64: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ResultOfGetBocHash {
    pub hash: String,
}

use ton_sdk;
use dispatch::DispatchTable;
use client::ClientContext;

pub(crate) fn encode_message_with_sign(_context: &mut ClientContext, params: ParamsOfEncodeMessageWithSign) -> ApiResult<EncodedMessage> {
    let (body, id) = ton_sdk::Contract::add_sign_to_message(
        &base64_decode(&params.signBytesBase64)?,
        &hex_decode(&params.publicKeyHex)?,
        &base64_decode(&params.unsignedBytesBase64)?
    ).map_err(|err|ApiError::contracts_encode_message_with_sign_failed(err))?;
    Ok(EncodedMessage {
        messageId: id.to_string(),
        messageIdBase64: id.to_base64().map_err(|err| ApiError::contracts_encode_message_with_sign_failed(err))?,
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

pub(crate) fn convert_address(_context: &mut ClientContext, params: ParamsOfConvertAddress) -> ApiResult<ResultOfConvertAddress> {
    debug!("-> contracts.image.code({}, {:?}, {:?})", params.address, params.convertTo, params.base64Params);
    let address = account_decode(&params.address)?;
    Ok(ResultOfConvertAddress {
        address: account_encode_ex(&address, params.convertTo, params.base64Params)?,
    })
}

pub(crate) fn get_boc_root_hash(_context: &mut ClientContext, params: ParamsOfGetBocHash) -> ApiResult<ResultOfGetBocHash> {
    debug!("-> contracts.boc.hash({})", params.bocBase64);
    let bytes = base64_decode(&params.bocBase64)?;
    let cells = ton_block::cells_serialization::deserialize_tree_of_cells(&mut bytes.as_slice())
        .map_err(|err| ApiError::contracts_invalid_boc(err))?;
    Ok(ResultOfGetBocHash {
        hash: format!("{:x}", cells.repr_hash()),
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
        |context, params| run::local_run(context, params, true));
    handlers.spawn("contracts.run.local.msg",
        |context, params| run::local_run_msg(context, params, true));
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
}
