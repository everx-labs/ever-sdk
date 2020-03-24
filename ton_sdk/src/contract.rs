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

use crate::*;
use ed25519_dalek::{Keypair, PublicKey};
use chrono::prelude::Utc;
use std::convert::{Into, TryFrom};
use std::io::{Cursor, Read, Seek};
use ton_block::{
    Account, AccountState, AccountStatus, AccountStorage, CurrencyCollection, Deserializable,
    ExternalInboundMessageHeader, GetRepresentationHash, Message as TvmMessage, MsgAddressInt,
    Serializable, StateInit, StorageInfo};
use ton_types::cells_serialization::{deserialize_cells_tree, BagOfCells};
use ton_types::{Cell, SliceData, HashmapE};
use ton_block::AccountId;

#[cfg(feature = "fee_calculation")]
use ton_executor::BlockchainConfig;

pub use ton_abi::json_abi::DecodedMessage;
pub use ton_abi::token::{Token, TokenValue, Tokenizer, Detokenizer};


#[cfg(feature = "node_interaction")]
use futures::{TryFutureExt, FutureExt};
#[cfg(feature = "node_interaction")]
use json_helper::account_status_to_u8;
#[cfg(feature = "node_interaction")]
use ton_block::TransactionProcessingStatus;
#[cfg(feature = "node_interaction")]
use std::collections::HashMap;
#[cfg(feature = "node_interaction")]
use std::iter::FromIterator;
#[cfg(feature = "node_interaction")]
use serde_json::Value;

#[cfg(feature = "node_interaction")]
const ACCOUNT_FIELDS: &str = r#"
    id
    acc_type
    balance
    balance_other {
        currency
        value
    }
    code
    data
    last_paid
"#;

// The struct represents value of some addititonal currency
#[derive(Deserialize, Default, Debug, Clone)]
pub struct OtherCurrencyValue {
    currency: u32,
    value: u128,
}

// The struct represents smart contract and allows
// to deploy and call it, to get some contract's properties.
// Don't forget - in TON blockchain Contract and Account are the same substances.
#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct Contract {
    #[serde(deserialize_with = "json_helper::deserialize_address_int_from_string")]
    pub id: MsgAddressInt,
    #[serde(deserialize_with = "json_helper::deserialize_account_status")]
    pub acc_type: AccountStatus,
    #[serde(deserialize_with = "json_helper::deserialize_uint_from_string")]
    pub balance: u128,
    pub balance_other: Option<Vec<OtherCurrencyValue>>,
    #[serde(deserialize_with = "json_helper::deserialize_tree_of_cells_opt_cell")]
    pub code: Option<Cell>,
    #[serde(deserialize_with = "json_helper::deserialize_tree_of_cells_opt_cell")]
    pub data: Option<Cell>,
    pub last_paid: u32,
}

// The struct represents conract's image
#[derive(Clone)]
pub struct ContractImage {
    state_init: StateInit,
    id: AccountId
}

#[allow(dead_code)]
impl ContractImage {

    // Creating contract image from code data and library bags of cells
    pub fn from_code_data_and_library<T>(code: &mut T, data: Option<&mut T>, library: Option<&mut T>) -> SdkResult<Self>
        where T: Read + Seek {

        let mut state_init = StateInit::default();

        let mut code_roots = deserialize_cells_tree(code)?;
        if code_roots.len() != 1 {
            bail!(SdkErrorKind::InvalidData { msg: "Invalid code's bag of cells".into() } );
        }
        state_init.set_code(code_roots.remove(0));

        if let Some(data_) = data {
            let mut data_roots = deserialize_cells_tree(data_)?;
            if data_roots.len() != 1 {
                bail!(SdkErrorKind::InvalidData { msg: "Invalid data's bag of cells".into() } );
            }
            state_init.set_data(data_roots.remove(0));
        }

        if let Some(library_) = library {
            let mut library_roots = deserialize_cells_tree(library_)?;
            if library_roots.len() != 1 {
                bail!(SdkErrorKind::InvalidData { msg: "Invalid library's bag of cells".into() } );
            }
            state_init.set_library(library_roots.remove(0));
        }

        let id = AccountId::from(state_init.hash()?);

        Ok(Self{ state_init, id })
    }

    pub fn new() -> SdkResult<Self> {
        let state_init = StateInit::default();
        let id = state_init.hash()?.into();

        Ok(Self{ state_init, id })
    }

    pub fn from_state_init<T>(state_init_bag: &mut T) -> SdkResult<Self>
        where T: Read {

        let mut si_roots = deserialize_cells_tree(state_init_bag)?;
        if si_roots.len() != 1 {
            bail!(SdkErrorKind::InvalidData { msg: "Invalid state init's bag of cells".into() } );
        }

        let state_init : StateInit
            = StateInit::construct_from(&mut SliceData::from(si_roots.remove(0)))?;

        let id = state_init.hash()?.into();

        Ok(Self{ state_init, id })
    }

    pub fn from_state_init_and_key<T>(state_init_bag: &mut T, pub_key: &PublicKey) -> SdkResult<Self>
        where T: Read {

        let mut result = Self::from_state_init(state_init_bag)?;
        result.set_public_key(pub_key)?;

        Ok(result)
    }

    pub fn set_public_key(&mut self, pub_key: &PublicKey) -> SdkResult<()> {
        let state_init = &mut self.state_init;

        let new_data = AbiContract::insert_pubkey(
            state_init.data.clone().unwrap_or_default().into(),
            pub_key.as_bytes(),
        )?;
        state_init.set_data(new_data.into_cell());

        self.id = state_init.hash()?.into();

        Ok(())
    }
    
    pub fn get_serialized_code(&self) -> SdkResult<Vec<u8>> {
        match &self.state_init.code {
            Some(cell) => {
                let mut data = Vec::new();
                let bag = BagOfCells::with_root(&cell);
                bag.write_to(&mut data, false)?;

                Ok(data)
            },
            None => bail!(SdkErrorKind::InvalidData { msg: "State init has no code".to_owned() } )
        }
    }

    pub fn get_serialized_data(&self) -> SdkResult<Vec<u8>> {
        match &self.state_init.data {
            Some(cell) => {
                let mut data = Vec::new();
                let bag = BagOfCells::with_root(&cell);
                bag.write_to(&mut data, false)?;

                Ok(data)
            },
            None => bail!(SdkErrorKind::InvalidData { msg: "State init has no data".to_owned() } )
        }
    }

    pub fn serialize(&self) -> SdkResult<Vec<u8>> {
        let cell = self.state_init.write_to_new_cell()?;

        let mut data = Vec::new();
        let bag = BagOfCells::with_root(&cell.into());
        bag.write_to(&mut data, false)?;

        Ok(data)
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
    pub fn update_data(&mut self, data_json: &str, abi_json: &str) -> SdkResult<()> {
        let new_data = ton_abi::json_abi::update_contract_data(
            abi_json,
            data_json,
            self.state_init.data.clone().unwrap_or_default().into())?;

        self.state_init.set_data(new_data.into_cell());
        self.id = self.state_init.hash()?.into();

        Ok(())
    }
}

pub fn decode_std_base64(data: &str) -> SdkResult<MsgAddressInt> {
    // conversion from base64url
    let data = data.replace('_', "/").replace('-', "+");

    let vec = base64::decode(&data)?;

    // check CRC and address tag
    let mut crc = crc_any::CRC::crc16xmodem();
    crc.digest(&vec[..34]);

    if crc.get_crc_vec_be() != &vec[34..36] || vec[0] & 0x3f != 0x11 {
        bail!(SdkErrorKind::InvalidArg { msg: data.to_owned() } );
    };

    Ok(MsgAddressInt::with_standart(None, vec[1] as i8, vec[2..34].into())?)
}

pub fn encode_base64(address: &MsgAddressInt, bounceable: bool, test: bool, as_url: bool) -> SdkResult<String> {
    if let MsgAddressInt::AddrStd(address) = address {
        let mut tag = if bounceable { 0x11 } else { 0x51 };
        if test { tag |= 0x80 };
        let mut vec = vec![tag];
        vec.extend_from_slice(&address.workchain_id.to_be_bytes());
        vec.append(&mut address.address.get_bytestring(0));

        let mut crc = crc_any::CRC::crc16xmodem();
        crc.digest(&vec);
        vec.extend_from_slice(&crc.get_crc_vec_be());

        let result = base64::encode(&vec);

        if as_url {
            Ok(result.replace('/', "_").replace('+', "-"))
        } else {
            Ok(result)
        }
    } else { bail!(SdkErrorKind::InvalidData { msg: "Non-std address".to_owned() } ) }
}

#[allow(dead_code)]
#[cfg(feature = "node_interaction")]
impl Contract {

    // Asynchronously loads a Contract instance or None if contract with given id is not exists
    pub async fn load(client: &NodeClient, address: &MsgAddressInt) -> SdkResult<Option<Contract>> {
        let id = address.to_string();

        client.load_record_fields(
            CONTRACTS_TABLE_NAME,
            &id,
            ACCOUNT_FIELDS)
                .await
                .and_then(|val| {
                    if val == serde_json::Value::Null {
                        Ok(None)
                    } else {
                        let acc: Contract = serde_json::from_value(val)
                            .map_err(|err| SdkErrorKind::InvalidData { msg: format!("error parsing account: {}", err) } )?;

                        Ok(Some(acc))
                    }
            })
    }

    // Asynchronously loads a Contract instance or None if contract with given id is not exists
    pub async fn load_wait_deployed(client: &NodeClient, address: &MsgAddressInt, timeout: Option<u32>)
    -> SdkResult<Contract>
    {
        client.wait_for(
            CONTRACTS_TABLE_NAME,
            &json!({
                "id": {
                    "eq": address.to_string()
                },
                "acc_type": { "eq": account_status_to_u8(AccountStatus::AccStateActive) }
            }).to_string(),
            ACCOUNT_FIELDS,
            timeout)
                .await
                .and_then(|val| {
                    let acc: Contract = serde_json::from_value(val)
                        .map_err(|err| SdkErrorKind::InvalidData { msg: format!("error parsing account: {}", err) } )?;
                    Ok(acc)
                })
    }

    // Asynchronously loads a Contract's json representation
    // or null if message with given id is not exists
    pub async fn load_json(client: &NodeClient, id: AccountId) -> SdkResult<String> {
        client.load_record_fields(CONTRACTS_TABLE_NAME, &id.to_hex_string(), ACCOUNT_FIELDS)
            .await
            .map(|val| val.to_string())
    }

    fn calc_timeout(timeout: u32, grow_rate: f32, try_index: u8) -> u32 {
       (timeout as f64 * grow_rate.powi(try_index as i32) as f64) as u32
    }

    // Add `expire` parameter to contract functions header
    fn make_expire_header(client: &NodeClient, abi: String, header: Option<String>, try_index: u8) -> SdkResult<(Option<String>, Option<u32>)> {
        let abi = AbiContract::load(abi.as_bytes())?;
        // use expire only if contract supports it
        if abi.header().contains(&serde_json::from_value::<ton_abi::Param>("expire".into())?) {
            // expire is `now + timeout`
            let timeout = Self::calc_timeout(
                client.get_timeouts().message_expiration_timeout,
                client.get_timeouts().message_expiration_timeout_grow_factor,
                try_index);
            let expire = Self::get_now()? + timeout / 1000;
            let expire = ton_abi::TokenValue::Expire(expire);

            let header = serde_json::from_str::<Value>(&header.unwrap_or("{}".to_owned()))?;
            // parse provided header using calculated value as default for expire param
            let header = Tokenizer::tokenize_optional_params(
                abi.header(),
                &header,
                &HashMap::from_iter(std::iter::once(("expire".to_owned(), expire))))?;
            // take resulting expire value to use it in transaction waiting
            let expire = match header.get("expire").unwrap() {
                TokenValue::Expire(expire) => *expire,
                _ => return Err(SdkErrorKind::InternalError{msg: "Wrong expire type".to_owned()}.into())
            };

            Ok((Some(Detokenizer::detokenize_optional(&header)?), Some(expire)))
        } else {
            Ok((header, None))
        }
    }

    const MESSAGE_EXPIRED_CODE: i32 = 57;
    const REPLAY_PROTECTION_CODE: i32 = 52;

    async fn retry_call<F, Fut>(retries_count: u8, func: F) -> SdkResult<Transaction>
    where
        F: Fn(u8) -> SdkResult<Fut>,
        Fut: futures::Future<Output=SdkResult<Transaction>>
    {
        for i in 0..(retries_count + 1) {
            //println!("Try#{}", i);
            let result = func(i)?.await;
            match &result {
                Err(error) => {
                    match error.downcast_ref::<SdkErrorKind>() {
                        Some(SdkErrorKind::MessageExpired) => continue,
                        _ => return result
                    }
                },
                _ => return result
            }
        }
        Err(SdkErrorKind::MessageExpired.into())
    }

    // Packs given inputs by abi and asynchronously calls contract.
    // Works with json representation of input and abi.
    // To get calling result - need to load message,
    // it's id and processing status is returned by this function
    pub async fn call_json(client: &NodeClient, address: MsgAddressInt, func: String, header: Option<String>, input: String,
        abi: String, key_pair: Option<&Keypair>
    ) -> SdkResult<Transaction> {
        Self::retry_call(client.get_timeouts().message_retries_count, |try_index: u8| {
            let (header, expire) = Self::make_expire_header(client, abi.clone(), header.clone(), try_index)?;

            // pack params into bag of cells via ABI
            let msg_body = ton_abi::encode_function_call(abi.clone(), func.clone(), header.clone(), input.clone(), false, key_pair)?;

            let msg = Self::create_message(address.clone(), msg_body.into())?;

            Ok(Self::send_message(client, msg, expire, try_index))
        }).await
    }

    // Packs given image and input and asynchronously calls given contract's constructor method.
    // Works with json representation of input and abi.
    // To get calling result - need to load message,
    // it's id and processing status is returned by this function
    pub async fn deploy_json(client: &NodeClient, func: String, header: Option<String>, input: String, abi: String,
        image: ContractImage, key_pair: Option<&Keypair>, workchain_id: i32
    ) -> SdkResult<Transaction> {
        Self::retry_call(client.get_timeouts().message_retries_count, |try_index: u8| {
            let (header, expire) = Self::make_expire_header(client, abi.clone(), header.clone(), try_index)?;

            let msg_body = ton_abi::encode_function_call(abi.clone(), func.clone(), header.clone(), input.clone(), false, key_pair)?;

            let cell = msg_body.into();
            let msg = Self::create_deploy_message(Some(cell), image.clone(), workchain_id)?;

            Ok(Self::send_message(client, msg, expire, try_index))
        }).await
    }

    // Packs given image asynchronously send deploy message into blockchain.
    // To get calling result - need to load message,
    // it's id and processing status is returned by this function
    pub async fn deploy_no_constructor(client: &NodeClient, image: ContractImage, workchain_id: i32)
        -> SdkResult<Transaction> {
            let msg = Self::create_deploy_message(None, image, workchain_id)?;

            Self::send_message(client, msg, None, 0).await
    }

    // Asynchronously calls contract by sending given message.
    // To get calling result - need to load message,
    // it's id and processing status is returned by this function
    pub async fn send_message(client: &NodeClient, msg: TvmMessage, expire: Option<u32>, try_index: u8)
        -> SdkResult<Transaction> 
    {
        // send message
        let msg_id = Self::_send_message(client, msg).await?;
        // subscribe on updates from DB and return updates stream
        Self::wait_transaction_processing(client, &msg_id, expire, try_index).await
    }

   async fn _send_message(client: &NodeClient, msg: TvmMessage) -> SdkResult<MessageId> {
        let (data, id) = Self::serialize_message(msg)?;
        client.send_message(&id.to_bytes()?, &data).await?;
        //println!("msg is sent, id: {}", id);
        Ok(id.clone())
    }

    pub async fn send_serialized_message(client: &NodeClient, id: &MessageId, msg: &[u8]) -> SdkResult<()> {
        client.send_message(&id.to_bytes()?, msg).await
    }

    pub async fn wait_transaction_processing(
        client: &NodeClient,
        message_id: &MessageId,
        expire: Option<u32>,
        try_index: u8
    ) -> SdkResult<Transaction>
    {
        let mut timeout = Self::calc_timeout(
            client.get_timeouts().message_processing_timeout,
            client.get_timeouts().message_processing_timeout_grow_factor,
            try_index);
        if let Some(expire) = expire {
            //println!("expire {}", expire);
            let now = Self::get_now()?;
            if expire <= now {
                return Err(SdkErrorKind::InvalidArg{ msg: "Message already expired".to_owned() }.into());
            }
            timeout = (expire - now) * 1000 + timeout;
        }

        let filter = json!({
            "in_msg": { "eq": message_id.to_string() },
            "status": { "eq": json_helper::transaction_status_to_u8(TransactionProcessingStatus::Finalized) }
        }).to_string();

        let transaction_stream = client.wait_for(
            TRANSACTIONS_TABLE_NAME,
            &filter, 
            TRANSACTION_FIELDS_ORDINARY,
            Some(timeout))
                .and_then(|value|  async move {
                        let transaction = serde_json::from_value::<Transaction>(value)?;
                        if  transaction.compute.exit_code == Some(Self::MESSAGE_EXPIRED_CODE) ||
                            transaction.compute.exit_code == Some(Self::REPLAY_PROTECTION_CODE)
                        {
                            Err(SdkErrorKind::MessageExpired.into())
                        } else {
                            Ok(transaction)
                        }
                });

        let filter = json!({
                "master": { "min_shard_gen_utime": { "ge": expire }}
            }).to_string();

        if expire.is_some() {
            let block_stream = client.wait_for(
                BLOCKS_TABLE_NAME,
                &filter, 
                "in_msg_descr { transaction_id }",
                Some(timeout))
                    .and_then(|value| async move {
                            match value["in_msg_descr"][0]["transaction_id"].as_str() {
                                None => Err(SdkErrorKind::InvalidData {
                                    msg: "Invalid block recieved: no transaction ID".to_owned()
                                }.into()),
                                Some(string) => {
                                    client.wait_for(
                                        TRANSACTIONS_TABLE_NAME,
                                        &json!({ "id": { "eq": string }}).to_string(), 
                                        "id",
                                        Some(timeout))
                                        .await?;
    
                                    Err(SdkErrorKind::MessageExpired.into())
                                }
                            }
                        });
            futures::pin_mut!(transaction_stream, block_stream);
            futures::select!{
                transaction = transaction_stream.fuse() => transaction,
                block = block_stream.fuse() => block,
            }
        } else {
            transaction_stream.await
        }
    }
}

pub struct MessageToSign {
    pub message: Vec<u8>,
    pub data_to_sign: Vec<u8>
}

#[cfg(feature = "fee_calculation")]
pub struct LocalCallResult {
    pub messages: Vec<Message>,
    pub fees: TransactionFees
}

impl Contract {
    /// Returns contract's address
    pub fn address(&self) -> MsgAddressInt {
        self.id.clone()
    }

    /// Returns contract's identifier
    pub fn id(&self) -> SdkResult<AccountId> {
        Ok(self.id.get_address())
    }

    /// Returns contract's balance in NANO grams
    pub fn balance_grams(&self) -> SdkResult<u128> {
        Ok(self.balance)
    }

    /// Returns contract's balance in NANO grams
    pub fn balance_other(&self) -> SdkResult<Vec<OtherCurrencyValue>> {
        Ok(self.balance_other.clone().unwrap_or_default())
    }

    // ------- Decoding functions -------

    /// Creates `Contract` struct by data from database
    pub fn from_json(json: &str) -> SdkResult<Self> {
        let acc: Contract = serde_json::from_str(json)?;

        Ok(acc)
    }

    /// Creates `Contract` struct by serialized contract's bag of cells
    pub fn from_bytes(boc: &[u8]) -> SdkResult<Self> {
        Self::from_cells(
            ton_types::cells_serialization::deserialize_tree_of_cells(&mut Cursor::new(boc))?
                .into()
        )
    }

    /// Creates `Contract` struct by deserialized contract's tree of cells
    pub fn from_cells(mut root_cell_slice: SliceData) -> SdkResult<Self> {
        let acc: ton_block::Account = ton_block::Account::construct_from(&mut root_cell_slice)?;
        if acc.is_none() {
            bail!(SdkErrorKind::InvalidData { msg: "Account is none.".into() } );
        }

        let mut balance_other = vec!();
        ton_types::HashmapType::iterate(
            &acc.get_balance().unwrap().other,
                &mut |ref mut key, ref mut value| -> SdkResult<bool> {
                let value: ton_block::VarUInteger32 = ton_block::VarUInteger32::construct_from(value)?;
                balance_other.push(OtherCurrencyValue {
                    currency: key.get_next_u32()?,
                    value: num_traits::ToPrimitive::to_u128(value.value()).ok_or(
                        SdkError::from(SdkErrorKind::InvalidData { msg: "Account's other currency balance is too big".to_owned() } )
                    )?
                });
                Ok(true)
            }
        ).unwrap();

        // All unwraps below won't panic because the account is checked for none.
        Ok(Contract {
            id: acc.get_addr().unwrap().clone(),
            acc_type: acc.status(),
            balance: num_traits::ToPrimitive::to_u128(
                acc.get_balance().unwrap().grams.value()).ok_or(
                    SdkError::from(SdkErrorKind::InvalidData {
                        msg: "Account's balance is too big".to_owned() 
                    } )
                )?,
            balance_other: if balance_other.len() > 0 { Some(balance_other) } else { None },
            code: acc.get_code(),
            data: acc.get_data(),
            last_paid: acc.storage_info().unwrap().last_paid,
        })
    }

    /// Invokes local TVM instance with provided inbound message.
    /// Returns outbound messages generated by contract function and gas fee function consumed
    pub fn local_call_tvm(&self, message: TvmMessage) -> SdkResult<Vec<Message>> {
        let code = self.code.clone().ok_or(
            SdkError::from(SdkErrorKind::InvalidData { msg: "Account has no code".to_owned() } ))?;
                
        let (tvm_messages, _) = local_tvm::call_tvm(
            self.balance,
            self.balance_other_as_hashmape()?,
            &self.id,
            None,
            Self::get_now()?,
            code,
            self.data.clone(),
            &message)?;

        let mut messages = vec![];
        for tvm_msg in &tvm_messages {
            messages.push(Message::with_msg(tvm_msg)?);
        }

        Ok(messages)
    }

    /// Invokes local TVM instance with provided inbound message.
    /// Returns outbound messages generated by contract function and gas fee function consumed
    pub fn local_call_tvm_json(&self, func: String, header: Option<String>, input: String,
        abi: String, key_pair: Option<&Keypair>
    ) -> SdkResult<Vec<Message>>
    {
        // pack params into bag of cells via ABI
        let msg_body = ton_abi::encode_function_call(abi, func, header, input, false, key_pair)?;

        let address = self.address();

        let msg = Self::create_message(address, msg_body.into())?;

        self.local_call_tvm(msg)
    }

    /// Invokes local transaction executor instance with provided inbound message.
    /// Returns outbound messages generated by contract function and transaction fees
    #[cfg(feature = "fee_calculation")]
    pub fn local_call(&self, message: TvmMessage) -> SdkResult<LocalCallResult> {
       // TODO: get real config
        let (tvm_messages, fees) = local_tvm::executor::call_executor(
            self.to_account()?,
            message,
            &BlockchainConfig::default(),
            Self::get_now()?)?;
                
        let mut messages = vec![];
        for tvm_msg in &tvm_messages {
            messages.push(Message::with_msg(tvm_msg)?);
        }

        Ok(LocalCallResult { messages, fees })
    }

    /// Invokes local transaction executor instance with provided inbound message.
    /// Returns outbound messages generated by contract function and transaction fees
    #[cfg(feature = "fee_calculation")]
    pub fn local_call_json(&self, func: String, header: Option<String>, input: String, abi: String, key_pair: Option<&Keypair>)
         -> SdkResult<LocalCallResult>
    {
        // pack params into bag of cells via ABI
        let msg_body = ton_abi::encode_function_call(abi, func, header, input, false, key_pair)?;

        let address = self.address();

        let msg = Self::create_message(address, msg_body.into())?;

        self.local_call(msg)
    }

    /// Decodes output parameters returned by contract function call 
    pub fn decode_function_response_json(abi: String, function: String, response: SliceData, internal: bool) 
        -> SdkResult<String> {

        ton_abi::json_abi::decode_function_response(abi, function, response, internal)
    }

    /// Decodes output parameters returned by contract function call from serialized message body
    pub fn decode_function_response_from_bytes_json(abi: String, function: String, response: &[u8], internal: bool)
        -> SdkResult<String> {

        let slice = Self::deserialize_tree_to_slice(response)?;

        Self::decode_function_response_json(abi, function, slice, internal)
    }

    /// Decodes output parameters returned by contract function call 
    pub fn decode_unknown_function_response_json(abi: String, response: SliceData, internal: bool) 
        -> SdkResult<DecodedMessage> {

        ton_abi::json_abi::decode_unknown_function_response(abi, response, internal)
    }

    /// Decodes output parameters returned by contract function call from serialized message body
    pub fn decode_unknown_function_response_from_bytes_json(abi: String, response: &[u8], internal: bool)
        -> SdkResult<DecodedMessage> {

        let slice = Self::deserialize_tree_to_slice(response)?;

        Self::decode_unknown_function_response_json(abi, slice, internal)
    }

    /// Decodes output parameters returned by contract function call 
    pub fn decode_unknown_function_call_json(abi: String, response: SliceData, internal: bool) 
        -> SdkResult<DecodedMessage> {

        ton_abi::json_abi::decode_unknown_function_call(abi, response, internal)
    }

    /// Decodes output parameters returned by contract function call from serialized message body
    pub fn decode_unknown_function_call_from_bytes_json(abi: String, response: &[u8], internal: bool)
        -> SdkResult<DecodedMessage> {

        let slice = Self::deserialize_tree_to_slice(response)?;

        Self::decode_unknown_function_call_json(abi, slice, internal)
    }

    // ------- Call constructing functions -------

    // Packs given inputs by abi into Message struct.
    // Works with json representation of input and abi.
    // Returns message's bag of cells and identifier.
    pub fn construct_call_message_json(address: MsgAddressInt, func: String, header: Option<String>,
        input: String, abi: String, internal: bool, key_pair: Option<&Keypair>
    ) -> SdkResult<(Vec<u8>, MessageId)> {

        // pack params into bag of cells via ABI
        let msg_body = ton_abi::encode_function_call(abi, func, header, input, internal, key_pair)?;

        let msg = Self::create_message(address, msg_body.into())?;

        Self::serialize_message(msg)
    }

    // Creates Message struct with provided body and account address
    // Returns message's bag of cells and identifier.
    pub fn construct_call_message_with_body(address: MsgAddressInt, body: &[u8]) -> SdkResult<(Vec<u8>, MessageId)> {
        let body_cell = Self::deserialize_tree_to_slice(body)?;

        let msg = Self::create_message(address, body_cell)?;

        Self::serialize_message(msg)
    }

    // Packs given inputs by abi into Message struct without sign and returns data to sign.
    // Sign should be then added with `add_sign_to_message` function
    // Works with json representation of input and abi.
    pub fn get_call_message_bytes_for_signing(address: MsgAddressInt, func: String, 
        header: Option<String>, input: String, abi: String
    ) -> SdkResult<MessageToSign> {
        
        // pack params into bag of cells via ABI
        let (msg_body, data_to_sign) = ton_abi::prepare_function_call_for_sign(abi, func, header, input)?;

        let msg = Self::create_message(address, msg_body.into())?;

        Self::serialize_message(msg).map(|(msg_data, _id)| {
                MessageToSign { message: msg_data, data_to_sign } 
            }
        )
    }

     // ------- Deploy constructing functions -------

    // Packs given image and input into Message struct.
    // Works with json representation of input and abi.
    // Returns message's bag of cells and identifier.
    pub fn construct_deploy_message_json(func: String, header: Option<String>, input: String,
        abi: String, image: ContractImage, key_pair: Option<&Keypair>, workchain_id: i32
    ) -> SdkResult<(Vec<u8>, MessageId)> {

        let msg_body = ton_abi::encode_function_call(abi, func, header, input, false, key_pair)?;

        let cell = msg_body.into();
        let msg = Self::create_deploy_message(Some(cell), image, workchain_id)?;

        Self::serialize_message(msg)
    }

    // Packs given image and body into Message struct.
    // Returns message's bag of cells and identifier.
    pub fn construct_deploy_message_with_body(image: ContractImage, body: Option<&[u8]>, workchain_id: i32) -> SdkResult<(Vec<u8>, MessageId)> {
        let body_cell = match body {
            None => None,
            Some(data) => Some(Self::deserialize_tree_to_slice(data)?)
        };

        let msg = Self::create_deploy_message(body_cell, image, workchain_id)?;
        
        Self::serialize_message(msg)
    }

    // Packs given image into Message struct.
    // Returns message's bag of cells and identifier.
    pub fn construct_deploy_message_no_constructor(image: ContractImage, workchain_id: i32)
        -> SdkResult<(Vec<u8>, MessageId)>
    {
        let msg = Self::create_deploy_message(None, image, workchain_id)?;

        Self::serialize_message(msg)
    }
    
    // Packs given image and input into Message struct without sign and returns data to sign.
    // Sign should be then added with `add_sign_to_message` function
    // Works with json representation of input and abi.
    pub fn get_deploy_message_bytes_for_signing(func: String, header: Option<String>, input: String,
        abi: String, image: ContractImage, workchain_id: i32
    ) -> SdkResult<MessageToSign> {

        let (msg_body, data_to_sign) = ton_abi::prepare_function_call_for_sign(abi, func, header, input)?;

        let cell = msg_body.into();
        let msg = Self::create_deploy_message(Some(cell), image, workchain_id)?;

        Self::serialize_message(msg).map(|(msg_data, _id)| {
                MessageToSign { message: msg_data, data_to_sign } 
            }
        )
    }


    // Add sign to message, returned by `get_deploy_message_bytes_for_signing` or 
    // `get_run_message_bytes_for_signing` function.
    // Returns serialized message and identifier.
    pub fn add_sign_to_message(abi: String, signature: &[u8], public_key: Option<&[u8]>, message: &[u8]) 
        -> SdkResult<(Vec<u8>, MessageId)> {
        
        let mut slice = Self::deserialize_tree_to_slice(message)?;

        let mut message: TvmMessage = TvmMessage::construct_from(&mut slice)?;

        let body = message.body()
            .ok_or(SdkError::from(SdkErrorKind::InvalidData { msg: "No message body".to_owned() } ))?;

        let signed_body = ton_abi::add_sign_to_function_call(abi, signature, public_key, body)?;

        *message.body_mut() = Some(signed_body.into());
            

        Self::serialize_message(message)
    }

    fn create_message(address: MsgAddressInt, msg_body: SliceData) -> SdkResult<TvmMessage> {

        let mut msg_header = ExternalInboundMessageHeader::default();
        msg_header.dst = address;
        
        let mut msg = TvmMessage::with_ext_in_header(msg_header);
        *msg.body_mut() = Some(msg_body);

        Ok(msg)
    }

    fn create_deploy_message(
        msg_body: Option<SliceData>,
        image: ContractImage,
        workchain_id: i32
    ) -> SdkResult<TvmMessage> {
        let mut msg_header = ExternalInboundMessageHeader::default();
        msg_header.dst = image.msg_address(workchain_id);
        let mut msg = TvmMessage::with_ext_in_header(msg_header);
        *msg.state_init_mut() = Some(image.state_init());
        *msg.body_mut() = msg_body;
        Ok(msg)
    }

    pub fn  serialize_message(msg: TvmMessage) -> SdkResult<(Vec<u8>, MessageId)> {
        let cells = msg.write_to_new_cell()?.into();

        let mut data = Vec::new();
        let bag = BagOfCells::with_root(&cells);
        bag.write_to(&mut data, false)?;

        Ok((data, (&cells.repr_hash().as_slice()[..]).into()))
    }

    /// Deserializes tree of cells from byte array into `SliceData`
    fn deserialize_tree_to_slice(data: &[u8]) -> SdkResult<SliceData> {
        let mut response_cells = deserialize_cells_tree(&mut Cursor::new(data))?;

        if response_cells.len() != 1 {
            bail!(SdkErrorKind::InvalidData { msg: "Deserialize message error".to_owned() } );
        }

        Ok(response_cells.remove(0).into())
    }

    /// Deserializes TvmMessage from byte array
    pub fn deserialize_message(message: &[u8]) -> SdkResult<TvmMessage> {
        let mut root_cells = deserialize_cells_tree(&mut Cursor::new(message))?;

        if root_cells.len() != 1 { 
            bail!(SdkErrorKind::InvalidData { msg: "Deserialize message error".to_owned() } );
        }

        Ok(TvmMessage::construct_from(&mut root_cells.remove(0).into())?)
    }

    fn balance_other_as_hashmape(&self) -> SdkResult<HashmapE> {
        let mut map = HashmapE::with_bit_len(32);

        if let Some(balance_vec) = &self.balance_other {
            for item in balance_vec {
                map.set(
                    item.currency.write_to_new_cell()?.into(),
                    &item.value.write_to_new_cell()?.into())?;
            }
        }

        Ok(map)
    }

    pub fn to_account(&self) -> SdkResult<Account> {
        let state = match &self.code {
            Some(code) => {
                let mut state_init = StateInit::default();
                state_init.code = Some(code.clone());
                state_init.data = self.data.clone();
                AccountState::with_state(state_init)
            },
            // account without code is considered uninit
            None => AccountState::AccountUninit
        };
        let storage = AccountStorage {
            last_trans_lt: 0,
            balance: CurrencyCollection { grams: self.balance.into(), other: self.balance_other_as_hashmape()? },
            state
        };
        Ok(Account::with_storage(
            &self.id,
            &StorageInfo::with_values(self.last_paid, None),
            &storage))
    }

    pub fn get_now() -> SdkResult<u32> {
        Ok(<u32>::try_from(Utc::now().timestamp())?)
    }
}


#[cfg(test)]
#[path = "tests/test_contract.rs"]
pub mod tests;
