/*
* Copyright 2018-2021 TON Labs LTD.
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

use crate::error::SdkError;
use crate::json_helper;
use crate::{AbiContract, MessageId};

use chrono::prelude::Utc;
use ed25519_dalek::{Keypair, PublicKey};
use serde_json::Value;
use std::convert::Into;
use std::io::{Read, Seek};
use ton_abi::json_abi::DecodedMessage;
use ton_block::{AccountIdPrefixFull, Deserializable, ExternalInboundMessageHeader, GetRepresentationHash,
    Message as TvmMessage, MsgAddressInt, Serializable, ShardIdent, StateInit,
    InternalMessageHeader, CurrencyCollection};
use ton_types::cells_serialization::deserialize_tree_of_cells;
use ton_types::{error, fail, AccountId, Result, SliceData};

pub struct Contract {}

#[derive(Clone, Debug)]
pub struct FunctionCallSet {
    pub func: String,
    pub header: Option<String>,
    pub input: String,
    pub abi: String,
}

pub struct SdkMessage {
    pub id: MessageId,
    pub serialized_message: Vec<u8>,
    pub message: TvmMessage,
    pub address: MsgAddressInt,
}

// The struct represents contract's image
#[derive(Clone)]
pub struct ContractImage {
    state_init: StateInit,
    id: AccountId,
}

#[allow(dead_code)]
impl ContractImage {
    // Creating contract image from code data and library bags of cells
    pub fn from_code_data_and_library<T>(
        code: &mut T,
        data: Option<&mut T>,
        library: Option<&mut T>,
    ) -> Result<Self>
    where
        T: Read + Seek,
    {
        let mut state_init = StateInit::default();

        state_init.set_code(deserialize_tree_of_cells(code)?);

        if let Some(data) = data {
            state_init.set_data(deserialize_tree_of_cells(data)?);
        }

        if let Some(library) = library {
            state_init.set_library(deserialize_tree_of_cells(library)?);
        }

        let id = AccountId::from(state_init.hash()?);

        Ok(Self { state_init, id })
    }

    pub fn new() -> Result<Self> {
        let state_init = StateInit::default();
        let id = state_init.hash()?.into();

        Ok(Self { state_init, id })
    }

    pub fn from_state_init<T>(state_init_bag: &mut T) -> Result<Self>
    where
        T: Read,
    {
        let cell = deserialize_tree_of_cells(state_init_bag)?;
        let state_init: StateInit = StateInit::construct_from_cell(cell)?;
        let id = state_init.hash()?.into();

        Ok(Self { state_init, id })
    }

    pub fn from_state_init_and_key<T>(state_init_bag: &mut T, pub_key: &PublicKey) -> Result<Self>
    where
        T: Read,
    {
        let mut result = Self::from_state_init(state_init_bag)?;
        result.set_public_key(pub_key)?;

        Ok(result)
    }

    pub fn from_cell(cell: ton_types::Cell) -> Result<Self> {
        let id = cell.repr_hash().into();
        let state_init = StateInit::construct_from_cell(cell)?;

        Ok(Self { state_init, id })
    }

    pub fn get_public_key(&self) -> Result<Option<PublicKey>> {
        let data = &self.state_init.data.as_ref().ok_or_else(
            || SdkError::InvalidData {
                msg: "State init has no data".to_owned()
            }
        )?.into();
        Ok(AbiContract::get_pubkey(data)?
            .map(|pub_key| PublicKey::from_bytes(&pub_key))
            .transpose()?
        )
    }

    pub fn set_public_key(&mut self, pub_key: &PublicKey) -> Result<()> {
        let state_init = &mut self.state_init;

        let new_data = AbiContract::insert_pubkey(
            state_init.data.clone().unwrap_or_default().into(),
            pub_key.as_bytes(),
        )?;
        state_init.set_data(new_data.into_cell());

        self.id = state_init.hash()?.into();

        Ok(())
    }

    pub fn get_serialized_code(&self) -> Result<Vec<u8>> {
        match &self.state_init.code {
            Some(cell) => ton_types::serialize_toc(cell),
            None => bail!(SdkError::InvalidData {
                msg: "State init has no code".to_owned()
            }),
        }
    }

    pub fn get_serialized_data(&self) -> Result<Vec<u8>> {
        match &self.state_init.data {
            Some(cell) => ton_types::serialize_toc(cell),
            None => bail!(SdkError::InvalidData {
                msg: "State init has no data".to_owned()
            }),
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        ton_types::serialize_toc(&self.state_init.serialize()?)
    }

    // Returns future contract's state_init struct
    pub fn state_init(self) -> StateInit {
        self.state_init
    }

    // Returns future contract's identifier
    pub fn account_id(&self) -> AccountId {
        self.id.clone()
    }

    // Returns future contract's address
    pub fn msg_address(&self, workchain_id: i32) -> MsgAddressInt {
        match workchain_id / 128 {
            0 => MsgAddressInt::with_standart(None, workchain_id as i8, self.id.clone()).unwrap(),
            _ => MsgAddressInt::with_variant(None, workchain_id, self.id.clone()).unwrap(),
        }
    }

    ///Allows to change initial values for public contract variables
    pub fn update_data(&mut self, data_json: &str, abi_json: &str) -> Result<()> {
        let new_data = ton_abi::json_abi::update_contract_data(
            abi_json,
            data_json,
            self.state_init.data.clone().unwrap_or_default().into(),
        )?;

        self.state_init.set_data(new_data.into_cell());
        self.id = self.state_init.hash()?.into();

        Ok(())
    }
}

pub struct MessageToSign {
    pub message: Vec<u8>,
    pub data_to_sign: Vec<u8>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ShardDescr {
    pub workchain_id: i32,
    #[serde(deserialize_with = "json_helper::deserialize_shard")]
    pub shard: u64,
}

impl Contract {
    /// Decodes output parameters returned by contract function call
    pub fn decode_function_response_json(
        abi: String,
        function: String,
        response: SliceData,
        internal: bool,
        allow_partial: bool,
    ) -> Result<String> {
        ton_abi::json_abi::decode_function_response(abi, function, response, internal, allow_partial)
    }

    /// Decodes output parameters returned by contract function call from serialized message body
    pub fn decode_function_response_from_bytes_json(
        abi: String,
        function: String,
        response: &[u8],
        internal: bool,
        allow_partial: bool,
    ) -> Result<String> {
        let slice = Self::deserialize_tree_to_slice(response)?;

        Self::decode_function_response_json(abi, function, slice, internal, allow_partial)
    }

    /// Decodes output parameters returned by contract function call
    pub fn decode_unknown_function_response_json(
        abi: String,
        response: SliceData,
        internal: bool,
        allow_partial: bool,
    ) -> Result<DecodedMessage> {
        ton_abi::json_abi::decode_unknown_function_response(abi, response, internal, allow_partial)
    }

    /// Decodes output parameters returned by contract function call from serialized message body
    pub fn decode_unknown_function_response_from_bytes_json(
        abi: String,
        response: &[u8],
        internal: bool,
        allow_partial: bool,
    ) -> Result<DecodedMessage> {
        let slice = Self::deserialize_tree_to_slice(response)?;

        Self::decode_unknown_function_response_json(abi, slice, internal, allow_partial)
    }

    /// Decodes output parameters returned by contract function call
    pub fn decode_unknown_function_call_json(
        abi: String,
        response: SliceData,
        internal: bool,
        allow_partial: bool,
    ) -> Result<DecodedMessage> {
        ton_abi::json_abi::decode_unknown_function_call(abi, response, internal, allow_partial)
    }

    /// Decodes output parameters returned by contract function call from serialized message body
    pub fn decode_unknown_function_call_from_bytes_json(
        abi: String,
        response: &[u8],
        internal: bool,
        allow_partial: bool,
    ) -> Result<DecodedMessage> {
        let slice = Self::deserialize_tree_to_slice(response)?;

        Self::decode_unknown_function_call_json(abi, slice, internal, allow_partial)
    }

    // ------- Call constructing functions -------

    // Packs given inputs by abi into an external inbound Message struct.
    // Works with json representation of input and abi.
    // Returns message's bag of cells and identifier.
    pub fn construct_call_ext_in_message_json(
        address: MsgAddressInt,
        params: FunctionCallSet,
        key_pair: Option<&Keypair>,
    ) -> Result<SdkMessage> {
        // pack params into bag of cells via ABI
        let msg_body = ton_abi::encode_function_call(
            params.abi,
            params.func,
            params.header,
            params.input,
            false,
            key_pair,
        )?;

        let msg = Self::create_ext_in_message(address.clone(), msg_body.into_cell()?.into())?;
        let (body, id) = Self::serialize_message(&msg)?;
        Ok(SdkMessage {
            id,
            serialized_message: body,
            message: msg,
            address,
        })
    }

    // Packs given inputs by abi into an internal Message struct.
    // Works with json representation of input and abi.
    // Returns message's bag of cells and identifier.
    pub fn construct_call_int_message_json(
        address: MsgAddressInt,
        src_address: Option<MsgAddressInt>,
        ihr_disabled: bool,
        bounce: bool,
        value: CurrencyCollection,
        params: FunctionCallSet,
    ) -> Result<SdkMessage> {
        // pack params into bag of cells via ABI
        let msg_body = ton_abi::encode_function_call(
            params.abi,
            params.func,
            None,
            params.input,
            true,
            None,
        )?;

        Self::construct_int_message_with_body(
            address,
            src_address,
            ihr_disabled,
            bounce,
            value,
            Some(msg_body.into_cell()?.into()),
        )
    }

    pub fn construct_int_message_with_body(
        dst_address: MsgAddressInt,
        src_address: Option<MsgAddressInt>,
        ihr_disabled: bool,
        bounce: bool,
        value: CurrencyCollection,
        msg_body: Option<SliceData>,
    ) -> Result<SdkMessage> {
        let msg = Self::create_int_message(ihr_disabled, bounce, dst_address.clone(), src_address, value, msg_body)?;
        let (body, id) = Self::serialize_message(&msg)?;
        Ok(SdkMessage {
            id,
            serialized_message: body,
            message: msg,
            address: dst_address,
        })
    }

    // Packs given inputs by abi into Message struct without sign and returns data to sign.
    // Sign should be then added with `add_sign_to_message` function
    // Works with json representation of input and abi.
    pub fn get_call_message_bytes_for_signing(
        address: MsgAddressInt,
        params: FunctionCallSet,
    ) -> Result<MessageToSign> {
        // pack params into bag of cells via ABI
        let (msg_body, data_to_sign) = ton_abi::prepare_function_call_for_sign(
            params.abi,
            params.func,
            params.header,
            params.input,
        )?;

        let msg = Self::create_ext_in_message(address, msg_body.into_cell()?.into())?;

        Self::serialize_message(&msg).map(|(msg_data, _id)| MessageToSign {
            message: msg_data,
            data_to_sign,
        })
    }

    // ------- Deploy constructing functions -------

    // Packs given image and input into Message struct.
    // Works with json representation of input and abi.
    // Returns message's bag of cells and identifier.
    pub fn construct_deploy_message_json(
        params: FunctionCallSet,
        image: ContractImage,
        key_pair: Option<&Keypair>,
        workchain_id: i32,
    ) -> Result<SdkMessage> {
        let msg_body = ton_abi::encode_function_call(
            params.abi,
            params.func,
            params.header,
            params.input,
            false,
            key_pair,
        )?;

        let cell: SliceData = msg_body.into_cell()?.into();
        let msg = Self::create_ext_deploy_message(Some(cell), image, workchain_id)?;

        let address = match msg.dst_ref() {
            Some(address) => address.clone(),
            None => fail!(SdkError::InternalError {
                msg: "No address in created deploy message".to_owned()
            })
        };
        let (body, id) = Self::serialize_message(&msg)?;

        Ok(SdkMessage {
            id,
            serialized_message: body,
            message: msg,
            address,
        })
    }

    // Packs given image and body into Message struct.
    // Returns message's bag of cells and identifier.
    pub fn construct_deploy_message_with_body(
        image: ContractImage,
        body: Option<&[u8]>,
        workchain_id: i32,
    ) -> Result<TvmMessage> {
        let body_cell = match body {
            None => None,
            Some(data) => Some(Self::deserialize_tree_to_slice(data)?),
        };

        Self::create_ext_deploy_message(body_cell, image, workchain_id)
    }

    // Packs given image into an external inbound Message struct.
    // Returns message's bag of cells and identifier.
    pub fn construct_deploy_message_no_constructor(
        image: ContractImage,
        workchain_id: i32,
    ) -> Result<TvmMessage> {
        Self::create_ext_deploy_message(None, image, workchain_id)
    }

    // Packs given image into an internal Message struct.
    // Returns message's bag of cells and identifier.
    pub fn construct_int_deploy_message_no_constructor(
        src: Option<MsgAddressInt>,
        image: ContractImage,
        workchain_id: i32,
        ihr_disabled: bool,
        bounce: bool,
    ) -> Result<TvmMessage> {
        Self::create_int_deploy_message(src, None, image, workchain_id, ihr_disabled, bounce)
    }

    // Packs given image and input into Message struct without signature and returns data to sign.
    // Signature should be then added with `add_sign_to_message` function
    // Works with json representation of input and abi.
    pub fn get_deploy_message_bytes_for_signing(
        params: FunctionCallSet,
        image: ContractImage,
        workchain_id: i32,
    ) -> Result<MessageToSign> {
        let (msg_body, data_to_sign) = ton_abi::prepare_function_call_for_sign(
            params.abi,
            params.func,
            params.header,
            params.input,
        )?;

        let cell: SliceData = msg_body.into_cell()?.into();
        let msg = Self::create_ext_deploy_message(Some(cell), image, workchain_id)?;

        Self::serialize_message(&msg).map(|(msg_data, _id)| MessageToSign {
            message: msg_data,
            data_to_sign,
        })
    }

    // Packs given image and input into Message struct with internal header and returns data.
    // Works with json representation of input and abi.
    pub fn get_int_deploy_message_bytes(
        src: Option<MsgAddressInt>,
        params: FunctionCallSet,
        image: ContractImage,
        workchain_id: i32,
        ihr_disabled: bool,
        bounce: bool,
    ) -> Result<Vec<u8>> {
        let msg_body = ton_abi::encode_function_call(
            params.abi,
            params.func,
            None,
            params.input,
            true,
            None,
        )?;

        let cell: SliceData = msg_body.into_cell()?.into();
        let msg = Self::create_int_deploy_message(src, Some(cell), image, workchain_id, ihr_disabled, bounce)?;

        Self::serialize_message(&msg)
            .map(|(msg_data, _id)| msg_data)
    }

    // Add sign to message, returned by `get_deploy_message_bytes_for_signing` or
    // `get_run_message_bytes_for_signing` function.
    // Returns serialized message and identifier.
    pub fn add_sign_to_message(
        abi: String,
        signature: &[u8],
        public_key: Option<&[u8]>,
        message: &[u8],
    ) -> Result<SdkMessage> {
        let mut slice = Self::deserialize_tree_to_slice(message)?;

        let mut message: TvmMessage = TvmMessage::construct_from(&mut slice)?;

        let body = message.body().ok_or(error!(SdkError::InvalidData {
            msg: "No message body".to_owned()
        }))?;

        let signed_body = ton_abi::add_sign_to_function_call(abi, signature, public_key, body)?;
        message.set_body(signed_body.into_cell()?.into());

        let address = match message.dst_ref() {
            Some(address) => address.clone(),
            None => fail!(SdkError::InternalError {
                msg: "No address in signed message".to_owned()
            })
        };
        let (body, id) = Self::serialize_message(&message)?;

        Ok(SdkMessage {
            id,
            address,
            serialized_message: body,
            message,
        })
    }

    // Add sign to message, returned by `get_deploy_message_bytes_for_signing` or
    // `get_run_message_bytes_for_signing` function.
    // Returns serialized message and identifier.
    pub fn attach_signature(
        abi: &AbiContract,
        signature: &[u8],
        public_key: Option<&[u8]>,
        message: &[u8],
    ) -> Result<SdkMessage> {
        let mut slice = Self::deserialize_tree_to_slice(message)?;

        let mut message: TvmMessage = TvmMessage::construct_from(&mut slice)?;

        let body = message.body().ok_or(error!(SdkError::InvalidData {
            msg: "No message body".to_owned()
        }))?;

        let signed_body = abi.add_sign_to_encoded_input(signature, public_key, body)?;
        message.set_body(signed_body.into_cell()?.into());

        let address = match message.dst_ref() {
            Some(address) => address.clone(),
            None => fail!(SdkError::InternalError{
                msg: "No address in signed message".to_owned()
            })
        };
        let (body, id) = Self::serialize_message(&message)?;

        Ok(SdkMessage {
            id,
            address,
            serialized_message: body,
            message,
        })
    }

    fn create_ext_in_message(address: MsgAddressInt, msg_body: SliceData) -> Result<TvmMessage> {
        let mut msg_header = ExternalInboundMessageHeader::default();
        msg_header.dst = address;

        let mut msg = TvmMessage::with_ext_in_header(msg_header);
        msg.set_body(msg_body);

        Ok(msg)
    }

    fn create_int_message(
        ihr_disabled: bool,
        bounce: bool,
        dst: MsgAddressInt,
        src: Option<MsgAddressInt>,
        value: CurrencyCollection,
        msg_body: Option<SliceData>,
    ) -> Result<TvmMessage> {
        let mut msg_header = InternalMessageHeader::default();
        if let Some(src) = src {
            msg_header.set_src(src);
        }
        msg_header.set_dst(dst);
        msg_header.value = value;
        msg_header.ihr_disabled = ihr_disabled;
        msg_header.bounce = bounce;
        let mut msg = TvmMessage::with_int_header(msg_header);
        msg_body.map(|body| msg.set_body(body));

        Ok(msg)
    }

    pub(crate) fn create_ext_deploy_message(
        msg_body: Option<SliceData>,
        image: ContractImage,
        workchain_id: i32,
    ) -> Result<TvmMessage> {
        let msg_header = ExternalInboundMessageHeader {
            dst: image.msg_address(workchain_id),
            ..Default::default()
        };

        let mut msg = TvmMessage::with_ext_in_header(msg_header);
        msg.set_state_init(image.state_init());
        msg_body.map(|body| msg.set_body(body));

        Ok(msg)
    }

    pub(crate) fn create_int_deploy_message(
        src: Option<MsgAddressInt>,
        msg_body: Option<SliceData>,
        image: ContractImage,
        workchain_id: i32,
        ihr_disabled: bool,
        bounce: bool,
    ) -> Result<TvmMessage> {
        let dst = image.msg_address(workchain_id);
        let mut msg_header = InternalMessageHeader::default();
        if let Some(src) = src {
            msg_header.set_src(src);
        }
        msg_header.set_dst(dst);
        msg_header.ihr_disabled = ihr_disabled;
        msg_header.bounce = bounce;

        let mut msg = TvmMessage::with_int_header(msg_header);
        msg.set_state_init(image.state_init());
        msg_body.map(|body| msg.set_body(body));

        Ok(msg)
    }

    pub fn serialize_message(msg: &TvmMessage) -> Result<(Vec<u8>, MessageId)> {
        let cells = msg.write_to_new_cell()?.into_cell()?;
        Ok((
            ton_types::serialize_toc(&cells)?,
            (&cells.repr_hash().as_slice()[..]).into(),
        ))
    }

    /// Deserializes tree of cells from byte array into `SliceData`
    pub fn deserialize_tree_to_slice(data: &[u8]) -> Result<SliceData> {
        Ok(deserialize_tree_of_cells(&mut data.clone())?.into())
    }

    pub fn get_dst_from_msg(msg: &[u8]) -> Result<MsgAddressInt> {
        match Contract::deserialize_message(msg)?.dst_ref() {
            Some(address) => Ok(address.clone()),
            None => fail!(SdkError::InvalidData {
                msg: "Wrong message type (extOut)".to_owned()
            })
        }
    }

    /// Deserializes TvmMessage from byte array
    pub fn deserialize_message(message: &[u8]) -> Result<TvmMessage> {
        TvmMessage::construct_from_bytes(message)
    }

    pub fn now() -> u32 {
        Utc::now().timestamp() as u32
    }

    pub fn check_shard_match(shard_descr: Value, address: &MsgAddressInt) -> Result<bool> {
        let descr: ShardDescr = serde_json::from_value(shard_descr)?;
        let ident = ShardIdent::with_tagged_prefix(descr.workchain_id, descr.shard)?;
        Ok(ident.contains_full_prefix(&AccountIdPrefixFull::prefix(address)?))
    }

    pub fn find_matching_shard(shards: &Vec<Value>, address: &MsgAddressInt) -> Result<Value> {
        for shard in shards {
            if Self::check_shard_match(shard.clone(), address)? {
                return Ok(shard.clone());
            }
        }
        Ok(Value::Null)
    }
}
