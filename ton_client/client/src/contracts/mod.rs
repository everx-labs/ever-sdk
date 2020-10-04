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

use crate::error::{ApiResult, ApiError};
use ton_sdk::{AbiContract, Contract, ContractImage, SdkMessage};
use ton_block::{CommonMsgInfo, Deserializable};
use ton_types::deserialize_tree_of_cells;
use std::io::Cursor;
use crate::encoding::{
    base64_decode,
    account_decode,
    account_encode_ex,
    AccountAddressType,
    Base64AddressParams,
};

use crate::crypto::internal::{
    decode_public_key
};

#[cfg(feature = "node_interaction")]
use ton_sdk::MessageProcessingState;
#[cfg(feature = "node_interaction")]
use ton_block::MsgAddressInt;

pub(crate) mod deploy;
pub(crate) mod run;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EncodedMessage {
    /// account address
    pub address: Option<String>,
    /// message id
    pub message_id: String,
    /// message boc (header+body)
    pub message_body_base64: String,
    /// message expiration timestamp - message will not be processed by the contract after message expiration time
    pub expire: Option<u32>,
}

impl EncodedMessage {
    #[cfg(feature = "node_interaction")]
    pub fn address(&self) -> ApiResult<MsgAddressInt> {
        match &self.address {
            Some(addr) => account_decode(addr),
            None => {
                let msg = base64_decode(&self.message_body_base64)?;
                Contract::get_dst_from_msg(&msg)
                    .map_err(|err| ApiError::invalid_params(
                        "message",
                        format!("cannot get target address: {}", err)))
            }
        }
    }

    #[cfg(feature = "node_interaction")]
    pub fn into_sdk_msg(self) -> ApiResult<SdkMessage> {
        let bytes = base64_decode(&self.message_body_base64)?;
        Ok(SdkMessage {
            address: self.address()?,
            id: self.message_id.into(),
            message: Contract::deserialize_message(&bytes)
                .map_err(|err| ApiError::invalid_params("message", format!("cannot parse BOC ({})", err)))?,
            serialized_message: bytes,
            expire: self.expire,
        })
    }

    pub fn from_sdk_msg(msg: SdkMessage) -> Self {
        EncodedMessage {
            message_id: msg.id.to_string(),
            address: Some(msg.address.to_string()),
            message_body_base64: base64::encode(&msg.serialized_message),
            expire: msg.expire,
        }
    }
}

#[doc(summary = "Method that sends the message to blockchain and waits for the result transaction")]
/// Method sends the previously created message,
/// waits for the result transaction
/// and decodes the parameters returned by the contract function
/// according to ABI
#[cfg(feature = "node_interaction")]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfProcessMessage {
    /// contract ABI
    pub abi: Option<serde_json::Value>,
    /// name of the called function
    pub function_name: Option<String>,
    /// structure, containing message boc and additional fields
    pub message: EncodedMessage,
    /// flag that enables/disables infinite waiting for network recovery and infinite waiting for shard blocks
    /// (in case of increasing time intervals between shard blocks)
    #[serde(default)]
    pub infinite_wait: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EncodedUnsignedMessage {
    /// part of message body without signature containing encoded function parameters
    pub unsigned_bytes_base64: String,
    /// bytes that must be signed with user key pair (see crypto.sign)
    pub bytes_to_sign_base64: String,
    /// message expiration timestamp - message will not be processed by the contract after message expiration time
    pub expire: Option<u32>,
}

#[doc(summary = "Method that prepares a signed message")]
/// Method takes unsigned message and other parameters
/// to construct a signed message, that is ready to be sent.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfEncodeMessageWithSign {
    /// contract ABI
    pub abi: serde_json::Value,
    /// part of message body without signature containing encoded function parameters
    pub unsigned_bytes_base64: String,
    /// signature
    pub sign_bytes_base64: String,
    /// public key
    pub public_key_hex: Option<String>,
    /// message expiration timestamp - message will not be processed by the contract after message expiration time
    pub expire: Option<u32>,
}

#[doc(summary="Method that calculates the function id for the specifiad ABI method")]
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ParamsOfGetFunctionId {
    /// contract ABI
    pub abi: serde_json::Value,
    /// function name
    pub function: String,
    /// specifies if the function id is calculated for contract function (true) or event/return (false)
    pub input: bool,
}

#[doc(summary = "Method that converts address to a specified format")]
/// method takes address in any TON address format: raw or user-friendly(base64
/// or base64url), and converts it into a specified format.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfConvertAddress {
    /// account address in any format
    pub address: String,
    /// specify the format to convert to
    pub convert_to: AccountAddressType,
    /// parameters of base64 format
    pub base64_params: Option<Base64AddressParams>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ResultOfGetFunctionId {
    // function id
    pub id: u32
}

#[doc(summary = "??? Method that extracts the code from a contract image.")]
/// Method takes the contract image (tvc converted to base64),
/// extracts the contract's boc with code and returns it.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfGetCodeFromImage {
    /// tvc converted to base64
    pub image_base64: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResultOfGetCodeFromImage {
    /// contract's boc with code in base64
    pub code_base64: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ResultOfConvertAddress {
    /// address in the specified format
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InputBoc {
    /// boc in base64
    pub boc_base64: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ResultOfGetBocHash {
    /// boc hash
    pub hash: String,
}

#[doc(summary = "Method that extracts the code from a contract image.")]
/// Method checks if the transaction is aborted or not and
/// - if aborted - exits with the exit_code from the transaction
/// - if not - checks if there is an output message generated by the function's return
/// operator, if it finds it - decodes it according to ABI and returns the result output.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfProcessTransaction {
    /// transaction in json format according to GraphQL schema
    pub transaction: serde_json::Value,
    /// contract ABI
    pub abi: Option<serde_json::Value>,
    /// function name
    pub function_name: Option<String>,
    /// account address
    pub address: String,
}

#[doc(summary = "Method that searches for the account shard")]
/// Method takes the list of shard hashes and searches
/// which shard the specified account belongs to
#[derive(Deserialize)]
pub(crate) struct ParamsOfFindShard {
    /// list of shards that are checked
    pub shards: Vec<serde_json::Value>,
    /// address which shard we look for
    pub address: String,
}

#[doc(summary = "Method that waits for the result transaction")]
/// Method waits for the result transaction generated by the message,
/// starting from message_processing_state.last_block_id
/// checks if there is an output message generated by the function's return
/// operator, if it finds it - decodes it according to ABI and returns the result output.
#[cfg(feature = "node_interaction")]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfWaitTransaction {
    /// contract ABI
    pub abi: Option<serde_json::Value>,
    /// contract's function name
    pub function_name: Option<String>,
    /// message object
    pub message: EncodedMessage,
    /// message processing state
    pub message_processing_state: MessageProcessingState,
    #[serde(default)]
    /// flag that enables/disables infinite waiting of the network recovery and infinite waiting of the shard blocks
    /// if set to false - method ends with exeption in case of network lags
    /// Lag maximum timeout is specified in Client Config:
    /// message_processing_timeout
    pub infinite_wait: bool,
}

use ton_sdk;
use crate::client::ClientContext;

pub(crate) fn encode_message_with_sign(_context: std::sync::Arc<ClientContext>, params: ParamsOfEncodeMessageWithSign) -> ApiResult<EncodedMessage> {
    let key_array: [u8; ed25519_dalek::PUBLIC_KEY_LENGTH];
    let public_key = if let Some(key) = params.public_key_hex {
        key_array = decode_public_key(&key)?.to_bytes();
        Some(key_array.as_ref())
    } else {
        None
    };
    let mut msg = ton_sdk::Contract::add_sign_to_message(
        params.abi.to_string(),
        &base64_decode(&params.sign_bytes_base64)?,
        public_key,
        &base64_decode(&params.unsigned_bytes_base64)?,
    ).map_err(|err| ApiError::contracts_encode_message_with_sign_failed(err))?;
    msg.expire = params.expire;

    Ok(EncodedMessage::from_sdk_msg(msg))
}

pub(crate) fn get_function_id(_context: std::sync::Arc<ClientContext>, params: ParamsOfGetFunctionId) -> ApiResult<ResultOfGetFunctionId> {
    let contract = AbiContract::load(params.abi.to_string().as_bytes())
        .map_err(|err| ApiError::contracts_get_function_id_failed(err, &params.function))?;

    let function = contract.function(&params.function)
        .map_err(|err| ApiError::contracts_get_function_id_failed(err, &params.function))?;

    Ok(ResultOfGetFunctionId {
        id: if params.input { function.get_input_id() } else { function.get_input_id() }
    })
}

pub(crate) fn get_code_from_image(_context: std::sync::Arc<ClientContext>, params: ParamsOfGetCodeFromImage) -> ApiResult<ResultOfGetCodeFromImage> {
    trace!("-> contracts.image.code()");

    let bytes = base64::decode(&params.image_base64)
        .map_err(|err| ApiError::contracts_invalid_image(err))?;
    let mut reader = Cursor::new(bytes);
    let image = ContractImage::from_state_init(&mut reader)
        .map_err(|err| ApiError::contracts_image_creation_failed(err))?;

    trace!("<-");
    Ok(ResultOfGetCodeFromImage {
        code_base64: base64::encode(&image.get_serialized_code()
            .map_err(|err| ApiError::contracts_image_creation_failed(err))?),
    })
}

pub(crate) fn convert_address(_context: std::sync::Arc<ClientContext>, params: ParamsOfConvertAddress) -> ApiResult<ResultOfConvertAddress> {
    trace!("-> contracts.image.code({}, {:?}, {:?})", params.address, params.convert_to, params.base64_params);
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

pub(crate) fn get_boc_root_hash(_context: std::sync::Arc<ClientContext>, params: InputBoc) -> ApiResult<ResultOfGetBocHash> {
    trace!("-> contracts.boc.hash({})", params.boc_base64);
    let cells = decode_boc_base64(&params.boc_base64)?;
    Ok(ResultOfGetBocHash {
        hash: format!("{:x}", cells.repr_hash()),
    })
}

#[cfg(feature = "node_interaction")]
pub(crate) async fn send_message(context: std::sync::Arc<ClientContext>, params: EncodedMessage) -> ApiResult<Option<MessageProcessingState>> {
    trace!("-> contracts.send.message({}, {})", params.message_id, params.expire.unwrap_or_default());

    let msg = base64_decode(&params.message_body_base64)?;
    let id = crate::encoding::hex_decode(&params.message_id)?;
    let client = context.get_sdk_client()?;
    let address = params.address()?;
    let is_old_client = params.address.is_none();
    let state = Contract::send_message(&client, &address, &id, &msg, params.expire)
        .await
        .map_err(|err| ApiError::contracts_send_message_failed(err))?;

    if is_old_client {
        Ok(None)
    } else {
        Ok(Some(state))
    }
}

#[cfg(feature = "node_interaction")]
pub(crate) async fn process_message(context: std::sync::Arc<ClientContext>, params: ParamsOfProcessMessage) -> ApiResult<run::ResultOfRun> {
    trace!("-> contracts.process.message({}, {})",
        params.message.message_id,
        params.message.expire.unwrap_or_default());

    let client = context.get_client()?;
    let sdk_client = context.get_sdk_client()?;
    let msg = params.message.into_sdk_msg()?;
    let result = Contract::process_message(
        sdk_client,
        &msg,
        params.infinite_wait)
        .await;

    let transaction = match result {
        Err(err) =>
            return Err(run::resolve_msg_sdk_error(
                client, sdk_client, err, &msg, params.function_name.as_ref().map(|string| string.as_str()), ApiError::contracts_process_message_failed,
            ).await?
            ),
        Ok(tr) => tr
    };

    run::process_transaction(
        transaction.parsed,
        transaction.value,
        params.abi,
        params.function_name,
        &msg.address,
        None,
        true)
}

pub(crate) fn process_transaction(
    _context: std::sync::Arc<ClientContext>, params: ParamsOfProcessTransaction,
) -> ApiResult<run::ResultOfRun> {
    trace!("-> contracts.process.transaction({}, {:?})", params.address, params.transaction);
    let address = account_decode(&params.address)?;
    let transaction = serde_json::from_value(params.transaction.clone())
        .map_err(|err| ApiError::invalid_params(&params.transaction.to_string(), err))?;

    run::process_transaction(
        transaction, params.transaction, params.abi, params.function_name, &address, None, true)
}

pub(crate) fn parse_message(_context: std::sync::Arc<ClientContext>, params: InputBoc) -> ApiResult<serde_json::Value> {
    trace!("-> contracts.boc.hash({})", params.boc_base64);
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

pub(crate) fn find_matching_shard(_context: std::sync::Arc<ClientContext>, params: ParamsOfFindShard) -> ApiResult<serde_json::Value> {
    trace!("-> contracts.find.shard({}, {:#?})", params.address, params.shards);
    let address = account_decode(&params.address)?;

    Contract::find_matching_shard(&params.shards, &address)
        .map_err(|err| ApiError::contracts_find_shard_failed(err))
}

#[cfg(feature = "node_interaction")]
pub(crate) async fn wait_transaction(context: std::sync::Arc<ClientContext>, params: ParamsOfWaitTransaction) -> ApiResult<run::ResultOfRun> {
    trace!("-> contracts.wait.transaction({}, {})",
        params.message.message_id,
        params.message.expire.unwrap_or_default());

    let client = context.get_client()?;
    let sdk_client = context.get_sdk_client()?;
    let msg = params.message.into_sdk_msg()?;
    let result = Contract::wait_transaction_processing(
        sdk_client,
        &msg.address,
        &msg.id,
        params.message_processing_state,
        msg.expire,
        params.infinite_wait)
        .await;

    let transaction = match result {
        Err(err) =>
            return Err(run::resolve_msg_sdk_error(
                client, sdk_client, err, &msg,
                params.function_name.as_ref().map(|string| string.as_str()),
                ApiError::contracts_process_message_failed,
            ).await?),
        Ok(tr) => tr
    };

    run::process_transaction(
        transaction.parsed,
        transaction.value,
        params.abi,
        params.function_name,
        &msg.address,
        None,
        true)
}


