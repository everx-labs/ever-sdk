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

use crate::json_helper;
use crate::local_tvm;
use crate::error::SdkError;
use crate::{AbiContract, BlockId, Message, MessageId, TimeoutsConfig, Transaction};

use ed25519_dalek::{Keypair, PublicKey};
use chrono::prelude::Utc;
use serde_json::Value;
use std::convert::{Into, TryFrom};
use std::io::{Cursor, Read, Seek};
use std::slice::Iter;
use std::collections::HashMap;
use std::sync::Arc;
use ton_block::{
    Account, AccountIdPrefixFull, AccountState, AccountStatus, AccountStorage, CurrencyCollection,
    Deserializable, ExternalInboundMessageHeader, GetRepresentationHash, Message as TvmMessage,
    MsgAddressInt, Serializable, ShardIdent, StateInit, StorageInfo};
use ton_types::cells_serialization::{deserialize_cells_tree, BagOfCells};
use ton_types::{error, fail, Result, AccountId, Cell, SliceData, HashmapE};
use ton_abi::json_abi::DecodedMessage;
use ton_abi::token::{Detokenizer, Tokenizer, TokenValue};
use ton_vm::stack::{StackItem, Stack};
use ton_vm::stack::integer::IntegerData;
use ton_executor::BlockchainConfig;

#[cfg(feature = "node_interaction")]
use crate::{
    NodeClient, Block,
    node_client::MAX_TIMEOUT,
    json_helper::account_status_to_u8,
    transaction::TRANSACTION_FIELDS_ORDINARY,
    types::{CONTRACTS_TABLE_NAME, TRANSACTIONS_TABLE_NAME},
};
#[cfg(feature = "node_interaction")]
use graphite::types::GraphiteError;

// JSON extension to StackItem
//
pub(crate) struct StackItemJSON;

impl StackItemJSON {
    fn invalid_json() -> SdkError {
        SdkError::InvalidData { msg: "Invalid JSON value for stack item".to_owned() }
    }

    pub(crate) fn json_array_from_items(items: Iter<StackItem>) -> Result<Value> {
        let mut values = Vec::<Value>::new();
        for item in items {
            values.push(StackItemJSON::json_value_from_item(item)?)
        }
        Ok(Value::Array(values))
    }

    pub(crate) fn items_from_json_array(values: Iter<Value>) -> Result<Vec<StackItem>> {
        let mut items = Vec::<StackItem>::new();
        for value in values {
            items.push(Self::item_from_json_value(value)?)
        }
        Ok(items)
    }

    fn json_value_from_item(item: &StackItem) -> Result<Value> {
        Ok(match item {
            StackItem::None =>
                Value::Null,
            StackItem::Integer(i) => {
                let mut hex = i.to_str_radix(16);
                if hex.ne("NaN") {
                    hex.insert_str(if hex.starts_with("-") { 1 } else { 0 }, "0x")
                }
                Value::String(hex)
            }
            StackItem::Tuple(items) =>
                Self::json_array_from_items(items.iter())?,
            StackItem::Builder(_) =>
                json!({"builder": Value::Null}),
            StackItem::Slice(_) =>
                json!({"slice": Value::Null}),
            StackItem::Cell(_) =>
                json!({"cell": Value::Null}),
            StackItem::Continuation(_) =>
                json!({"continuation": Value::Null}),
        })
    }

    fn parse_integer_data(s: &String) -> Result<IntegerData> {
        Ok(if s.eq("NaN") {
            IntegerData::nan()
        } else {
            let without_hex_prefix = s.replace("0x", "").replace("0X", "");
            IntegerData::from_str_radix(
                without_hex_prefix.as_str(),
                if s.len() == without_hex_prefix.len() { 10 } else { 16 },
            )?
        })
    }

    fn item_from_json_value(value: &Value) -> Result<StackItem> {
        Ok(match value {
            Value::Null =>
                StackItem::None,
            Value::Bool(v) =>
                StackItem::Integer(Arc::new(if *v {
                    IntegerData::one()
                } else {
                    IntegerData::zero()
                })),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    StackItem::Integer(Arc::new(IntegerData::from_i64(i)))
                } else {
                    return Err(error!(Self::invalid_json()));
                }
            }
            Value::String(s) => {
                StackItem::Integer(Arc::new(Self::parse_integer_data(s)?))
            }
            Value::Array(array) => {
                StackItem::Tuple(Self::items_from_json_array(array.iter())?)
            }
            Value::Object(_) =>
                return Err(error!(Self::invalid_json())),
        })
    }
}

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
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct OtherCurrencyValue {
    currency: u32,
    #[serde(with = "json_helper::uint")]
    value: u64,
}

// The struct represents smart contract and allows
// to deploy and call it, to get some contract's properties.
// Don't forget - in TON blockchain Contract and Account are the same substances.
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct Contract {
    #[serde(with = "json_helper::address")]
    pub id: MsgAddressInt,
    #[serde(with = "json_helper::account_status")]
    pub acc_type: AccountStatus,
    #[serde(with = "json_helper::uint")]
    pub balance: u64,
    pub balance_other: Option<Vec<OtherCurrencyValue>>,
    #[serde(with = "json_helper::opt_cell")]
    pub code: Option<Cell>,
    #[serde(with = "json_helper::opt_cell")]
    pub data: Option<Cell>,
    pub last_paid: u32,
}

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
    pub expire: Option<u32>,
    pub address: MsgAddressInt,
}

// The struct represents conract's image
#[derive(Clone)]
pub struct ContractImage {
    state_init: StateInit,
    id: AccountId,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageProcessingState {
    last_block_id: BlockId,
    sent_time: u32,
}

#[derive(Debug)]
pub struct RecievedTransaction {
    pub value: Value,
    pub parsed: Transaction
}

#[allow(dead_code)]
impl ContractImage {
    // Creating contract image from code data and library bags of cells
    pub fn from_code_data_and_library<T>(code: &mut T, data: Option<&mut T>, library: Option<&mut T>) -> Result<Self>
        where T: Read + Seek {
        let mut state_init = StateInit::default();

        let mut code_roots = deserialize_cells_tree(code)?;
        if code_roots.len() != 1 {
            bail!(SdkError::InvalidData { msg: "Invalid code's bag of cells".into() } );
        }
        state_init.set_code(code_roots.remove(0));

        if let Some(data_) = data {
            let mut data_roots = deserialize_cells_tree(data_)?;
            if data_roots.len() != 1 {
                bail!(SdkError::InvalidData { msg: "Invalid data's bag of cells".into() } );
            }
            state_init.set_data(data_roots.remove(0));
        }

        if let Some(library_) = library {
            let mut library_roots = deserialize_cells_tree(library_)?;
            if library_roots.len() != 1 {
                bail!(SdkError::InvalidData { msg: "Invalid library's bag of cells".into() } );
            }
            state_init.set_library(library_roots.remove(0));
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
        where T: Read {
        let mut si_roots = deserialize_cells_tree(state_init_bag)?;
        if si_roots.len() != 1 {
            bail!(SdkError::InvalidData { msg: "Invalid state init's bag of cells".into() } );
        }

        let state_init: StateInit
            = StateInit::construct_from(&mut SliceData::from(si_roots.remove(0)))?;

        let id = state_init.hash()?.into();

        Ok(Self { state_init, id })
    }

    pub fn from_state_init_and_key<T>(state_init_bag: &mut T, pub_key: &PublicKey) -> Result<Self>
        where T: Read {
        let mut result = Self::from_state_init(state_init_bag)?;
        result.set_public_key(pub_key)?;

        Ok(result)
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
            Some(cell) => {
                let mut data = Vec::new();
                let bag = BagOfCells::with_root(&cell);
                bag.write_to(&mut data, false)?;

                Ok(data)
            }
            None => bail!(SdkError::InvalidData { msg: "State init has no code".to_owned() } )
        }
    }

    pub fn get_serialized_data(&self) -> Result<Vec<u8>> {
        match &self.state_init.data {
            Some(cell) => {
                let mut data = Vec::new();
                let bag = BagOfCells::with_root(&cell);
                bag.write_to(&mut data, false)?;

                Ok(data)
            }
            None => bail!(SdkError::InvalidData { msg: "State init has no data".to_owned() } )
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
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
    pub fn update_data(&mut self, data_json: &str, abi_json: &str) -> Result<()> {
        let new_data = ton_abi::json_abi::update_contract_data(
            abi_json,
            data_json,
            self.state_init.data.clone().unwrap_or_default().into())?;

        self.state_init.set_data(new_data.into_cell());
        self.id = self.state_init.hash()?.into();

        Ok(())
    }
}

#[allow(dead_code)]
#[cfg(feature = "node_interaction")]
impl Contract {
    // Asynchronously loads a Contract instance or None if contract with given id is not exists
    pub async fn load(client: &NodeClient, address: &MsgAddressInt) -> Result<Option<Contract>> {
        let id = address.to_string();

        let value = client.load_record_fields(
            CONTRACTS_TABLE_NAME,
            &id,
            ACCOUNT_FIELDS).await?;

        if value == serde_json::Value::Null {
            Ok(None)
        } else {
            Ok(Some(serde_json::from_value(value)
                .map_err(|err| SdkError::InvalidData {
                    msg: format!("error parsing account: {}", err)
                })?))
        }
    }

    // Asynchronously loads a Contract instance or None if contract with given id is not exists
    pub async fn load_wait(
        client: &NodeClient, address: &MsgAddressInt, deployed: bool, timeout: Option<u32>
    ) -> Result<Contract> {
        let mut filter = json!({
            "id": {
                "eq": address.to_string()
            }
        });
        if deployed {
            filter["acc_type"] = json!({ "eq": account_status_to_u8(AccountStatus::AccStateActive)});
        }
        let value = client.wait_for(
            CONTRACTS_TABLE_NAME,
            &filter.to_string(),
            ACCOUNT_FIELDS,
            timeout).await?;

        serde_json::from_value(value)
            .map_err(|err| SdkError::InvalidData {
                msg: format!("error parsing account: {}", err)
            }.into())
    }

    // Asynchronously loads a Contract's json representation
    // or null if message with given id is not exists
    pub async fn load_json(client: &NodeClient, id: AccountId) -> Result<String> {
        client.load_record_fields(CONTRACTS_TABLE_NAME, &id.to_hex_string(), ACCOUNT_FIELDS)
            .await
            .map(|val| val.to_string())
    }

    pub async fn process_message(
        client: &NodeClient,
        msg: &SdkMessage,
        infinite_wait: bool
    ) -> Result<RecievedTransaction> {
        let state = Self::send_message(
            client, &msg.address, &msg.id.to_bytes()?, &msg.serialized_message, msg.expire).await?;
        //println!("msg is sent, id: {}", id);
        Self::wait_transaction_processing(
            client, &msg.address, &msg.id, state, msg.expire, infinite_wait).await
    }

    pub async fn send_message(
        client: &NodeClient,
        address: &MsgAddressInt,
        id: &[u8],
        msg: &[u8],
        expire: Option<u32>
    ) -> Result<MessageProcessingState> {
        let now = Self::now();
        if let Some(expire) = expire {
            if expire <= now {
                fail!(SdkError::InvalidArg { msg: "Message already expired".to_owned() });
            }
        }
        let block_id = Block::find_last_shard_block(client, address).await?;
        client.send_message(id, msg).await?;
        Ok(MessageProcessingState {
            sent_time: Self::now(),
            last_block_id: block_id,
        })
    }

    const MESSAGE_EXPIRED_CODE: i32 = 57;
    const REPLAY_PROTECTION_CODE: i32 = 52;

    pub async fn wait_transaction_processing(
        client: &NodeClient,
        address: &MsgAddressInt,
        message_id: &MessageId,
        mut state: MessageProcessingState,
        expire: Option<u32>,
        infinite_wait: bool
    ) -> Result<RecievedTransaction>
    {
        let stop_time = match expire {
            Some(expire) => expire,
            None => Self::now() + client.timeouts().message_processing_timeout / 1000
        };

        let mut transaction = Value::Null;
        let add_timeout = client.timeouts().message_processing_timeout;
        loop {
            let now = Self::now();
            let timeout = std::cmp::max(stop_time, now) - now + add_timeout;
            let result = Block::wait_next_block(client, &state.last_block_id, &address, Some(timeout)).await;
            let block = match result {
                Err(err) => {
                    if let Some(&SdkError::WaitForTimeout) = err.downcast_ref::<SdkError>() {
                        if infinite_wait {
                            continue;
                        } else {
                            fail!(SdkError::NetworkSilent {
                                msg_id: message_id.clone(),
                                block_id: state.last_block_id.clone(),
                                timeout,
                                state
                            });
                        }
                    } else if let Some(GraphiteError::NetworkError(_)) = err.downcast_ref::<GraphiteError>() {
                        if infinite_wait {
                            continue;
                        } else {
                            fail!(err);
                        }
                    } else {
                        fail!(SdkError::ResumableNetworkError {
                            state,
                            error: err
                        });
                    }
                }
                Ok(block) => block
            };

            state.last_block_id = block.id;

            for block_msg in &block.in_msg_descr {
                if Some(message_id) == block_msg.msg_id.as_ref() {
                    let tr_id = block_msg.transaction_id.clone()
                        .ok_or(SdkError::InvalidData {
                            msg: "No field `transaction_id` in block".to_owned() })?;

                    transaction = client.wait_for(
                        TRANSACTIONS_TABLE_NAME,
                        &json!({
                            "id": { "eq": tr_id.to_string() }
                        }).to_string(),
                        TRANSACTION_FIELDS_ORDINARY,
                        Some(MAX_TIMEOUT))
                        .await?;

                    break;
                }
            }
            if !transaction.is_null() {
                break;
            }

            if block.gen_utime > stop_time {
                if expire.is_some() {
                    fail!(SdkError::MessageExpired{
                        msg_id: message_id.clone(),
                        send_time: state.sent_time,
                        expire: stop_time,
                        block_time: block.gen_utime,
                        block_id: state.last_block_id
                    });
                } else {
                    fail!(SdkError::TransactionWaitTimeout {
                        msg_id: message_id.clone(),
                        send_time: state.sent_time,
                        timeout,
                        state
                    });
                }
                
            }
        }

        //println!("transaction recieved {:#}", transaction);
        let parsed = serde_json::from_value::<Transaction>(transaction.clone())?;
        if parsed.compute.exit_code == Some(Self::MESSAGE_EXPIRED_CODE) ||
        parsed.compute.exit_code == Some(Self::REPLAY_PROTECTION_CODE)
        {
            Err(SdkError::MessageExpired{
                msg_id: message_id.clone(),
                send_time: state.sent_time,
                expire: expire.unwrap_or(0),
                block_time: parsed.now,
                block_id: transaction["block_id"].as_str().unwrap_or("null").into()
            }.into())
        } else {
            Ok(RecievedTransaction { 
                parsed,
                value: transaction
            })
        }
    }
}

pub struct MessageToSign {
    pub message: Vec<u8>,
    pub data_to_sign: Vec<u8>,
    pub expire: Option<u32>,
}

pub struct LocalCallResult {
    pub transaction: Transaction,
    pub updated_account: Contract,
    pub updated_account_root: Cell
}

#[derive(Deserialize, Debug, Clone)]
pub struct ShardDescr {
    pub workchain_id: i32,
    #[serde(deserialize_with = "json_helper::deserialize_shard")]
    pub shard: u64,
}

impl Contract {
    /// Returns contract's address
    pub fn address(&self) -> MsgAddressInt {
        self.id.clone()
    }

    /// Returns contract's identifier
    pub fn id(&self) -> Result<AccountId> {
        Ok(self.id.get_address())
    }

    /// Returns contract's balance in NANO grams
    pub fn balance_grams(&self) -> Result<u64> {
        Ok(self.balance)
    }

    /// Returns contract's balance in NANO grams
    pub fn balance_other(&self) -> Result<Vec<OtherCurrencyValue>> {
        Ok(self.balance_other.clone().unwrap_or_default())
    }

    // ------- Decoding functions -------

    /// Creates `Contract` struct by data from database
    pub fn from_json(json: &str) -> Result<Self> {
        let acc: Contract = serde_json::from_str(json)?;

        Ok(acc)
    }

    /// Creates `Contract` struct by serialized contract's bag of cells
    pub fn from_bytes(boc: &[u8]) -> Result<Self> {
        Self::from_cells(
            ton_types::cells_serialization::deserialize_tree_of_cells(&mut Cursor::new(boc))?
                .into()
        )
    }

    /// Creates `Contract` struct by deserialized contract's tree of cells
    pub fn from_cells(mut root_cell_slice: SliceData) -> Result<Self> {
        let acc: ton_block::Account = ton_block::Account::construct_from(&mut root_cell_slice)?;
        if acc.is_none() {
            bail!(SdkError::InvalidData { msg: "Account is none.".into() } );
        }

        let mut balance_other = vec!();
        &acc.get_balance().unwrap().other.iterate_with_keys(
            |currency, value| -> Result<bool> {
                balance_other.push(OtherCurrencyValue {
                    currency,
                    value: num_traits::ToPrimitive::to_u64(value.value()).ok_or(
                        error!(SdkError::InvalidData { msg: "Account's other currency balance is too big".to_owned() })
                    )?,
                });
                Ok(true)
            }).unwrap();

        // All unwraps below won't panic because the account is checked for none.
        Ok(Contract {
            id: acc.get_addr().unwrap().clone(),
            acc_type: acc.status(),
            balance: num_traits::ToPrimitive::to_u64(
                acc.get_balance().unwrap().grams.value()).ok_or(
                error!(SdkError::InvalidData {
                    msg: "Account's balance is too big".to_owned()
                })
            )?,
            balance_other: if balance_other.len() > 0 { Some(balance_other) } else { None },
            code: acc.get_code(),
            data: acc.get_data(),
            last_paid: acc.storage_info().unwrap().last_paid,
        })
    }

    /// Invokes local TVM instance with provided inbound message.
    /// Returns outbound messages generated by contract function and gas fee function consumed
    pub fn local_call_tvm(&self, message: TvmMessage) -> Result<Vec<Message>> {
        let code = self.code.clone().ok_or(
            error!(SdkError::InvalidData { msg: "Account has no code".to_owned() }))?;

        let (tvm_messages, _) = local_tvm::call_tvm(
            self.balance,
            self.balance_other_as_hashmape()?,
            &self.id,
            None,
            Self::now(),
            code,
            self.data.clone(),
            &message)?;

        let mut messages = vec![];
        for tvm_msg in &tvm_messages {
            messages.push(Message::with_msg(tvm_msg)?);
        }

        Ok(messages)
    }


    /// Invokes local TVM instance with provided stack.
    /// Returns stack after contract execution.
    /// Used for get methods
    pub fn local_call_tvm_get_json(
        &self,
        function_name: &str,
        input: Option<&Value>,
    ) -> Result<Value> {
        let code = self.code.clone().ok_or(
            error!(SdkError::InvalidData { msg: "Account has no code".to_owned() }))?;
        let mut crc = crc_any::CRC::crc16xmodem();
        crc.digest(function_name.as_bytes());
        let function_id = ((crc.get_crc() as u32) & 0xffff) | 0x10000;
        let mut stack_in = Stack::new();
        if let Some(input) = input {
            if let Value::Array(array) = input {
                for value in array.iter() {
                    stack_in.push(StackItemJSON::item_from_json_value(value)?);
                }
            } else {
                stack_in.push(StackItemJSON::item_from_json_value(input)?);
            }
        }
        stack_in.push(StackItem::Integer(Arc::new(IntegerData::from_u32(function_id))));
        let stack_out = local_tvm::call_tvm_stack(
            self.balance,
            self.balance_other_as_hashmape()?,
            &self.id,
            None,
            Self::now(),
            code,
            self.data.clone(),
            stack_in)?;
        StackItemJSON::json_array_from_items(stack_out.iter())
    }

    /// Invokes local TVM instance with provided inbound message.
    /// Returns outbound messages generated by contract function and gas fee function consumed
    pub fn local_call_tvm_json(&self, func: String, header: Option<String>, input: String,
        abi: String, key_pair: Option<&Keypair>,
    ) -> Result<Vec<Message>>
    {
        // pack params into bag of cells via ABI
        let msg_body = ton_abi::encode_function_call(abi, func, header, input, false, key_pair)?;

        let address = self.address();

        let msg = Self::create_message(address, msg_body.into())?;

        self.local_call_tvm(msg)
    }

    /// Invokes local transaction executor instance with provided inbound message.
    /// Returns outbound messages generated by contract function and transaction fees
    pub fn local_call(&self, message: TvmMessage, time: Option<u32>) -> Result<LocalCallResult> {
        // TODO: get real config
        let (transaction, account_root) = local_tvm::executor::call_executor(
            self.to_account()?,
            message,
            BlockchainConfig::default(),
            time.unwrap_or(Self::now()))?;

        let transaction = Transaction::try_from(transaction)?;
        let updated_account = Self::from_cells(account_root.clone().into())?;
        Ok(LocalCallResult {
            transaction,
            updated_account,
            updated_account_root: account_root
        })
    }

    /// Invokes local transaction executor instance with provided inbound message.
    /// Returns outbound messages generated by contract function and transaction fees
    pub fn local_call_json(
        &self,
        func: String,
        header: Option<String>,
        input: String,
        abi: String,
        key_pair: Option<&Keypair>,
        time: Option<u32>
    ) -> Result<LocalCallResult> {
        // pack params into bag of cells via ABI
        let msg_body = ton_abi::encode_function_call(abi, func, header, input, false, key_pair)?;

        let address = self.address();

        let msg = Self::create_message(address, msg_body.into())?;

        self.local_call(msg, time)
    }

    /// Decodes output parameters returned by contract function call
    pub fn decode_function_response_json(abi: String, function: String, response: SliceData, internal: bool)
        -> Result<String> {
        ton_abi::json_abi::decode_function_response(abi, function, response, internal)
    }

    /// Decodes output parameters returned by contract function call from serialized message body
    pub fn decode_function_response_from_bytes_json(abi: String, function: String, response: &[u8], internal: bool)
        -> Result<String> {
        let slice = Self::deserialize_tree_to_slice(response)?;

        Self::decode_function_response_json(abi, function, slice, internal)
    }

    /// Decodes output parameters returned by contract function call
    pub fn decode_unknown_function_response_json(abi: String, response: SliceData, internal: bool)
        -> Result<DecodedMessage> {
        ton_abi::json_abi::decode_unknown_function_response(abi, response, internal)
    }

    /// Decodes output parameters returned by contract function call from serialized message body
    pub fn decode_unknown_function_response_from_bytes_json(abi: String, response: &[u8], internal: bool)
        -> Result<DecodedMessage> {
        let slice = Self::deserialize_tree_to_slice(response)?;

        Self::decode_unknown_function_response_json(abi, slice, internal)
    }

    /// Decodes output parameters returned by contract function call
    pub fn decode_unknown_function_call_json(abi: String, response: SliceData, internal: bool)
        -> Result<DecodedMessage> {
        ton_abi::json_abi::decode_unknown_function_call(abi, response, internal)
    }

    /// Decodes output parameters returned by contract function call from serialized message body
    pub fn decode_unknown_function_call_from_bytes_json(abi: String, response: &[u8], internal: bool)
        -> Result<DecodedMessage> {
        let slice = Self::deserialize_tree_to_slice(response)?;

        Self::decode_unknown_function_call_json(abi, slice, internal)
    }

    // ------- Call constructing functions -------

    // Calculate timeout according to try number and timeout grow rate
    // (timeouts are growing from try to try)
    fn calc_timeout(timeout: u32, grow_rate: f32, try_index: u8) -> u32 {
        (timeout as f64 * grow_rate.powi(try_index as i32) as f64) as u32
    }

    // Add `expire` parameter to contract functions header
    fn make_expire_header(
        timeouts: Option<&TimeoutsConfig>,
        abi: String,
        header: Option<String>,
        try_index: Option<u8>,
    ) -> Result<(Option<String>, Option<u32>)> {
        let abi = AbiContract::load(abi.as_bytes())?;
        // use expire only if contract supports it
        if abi.header().contains(&serde_json::from_value::<ton_abi::Param>("expire".into())?) {
            let default = TimeoutsConfig::default();
            let timeouts = timeouts.unwrap_or(&default);
            let try_index = try_index.unwrap_or(0);
            // expire is `now + timeout`
            let timeout = Self::calc_timeout(
                timeouts.message_expiration_timeout,
                timeouts.message_expiration_timeout_grow_factor,
                try_index);
            let expire = Self::now() + timeout / 1000;

            let mut header = serde_json::from_str::<Value>(&header.unwrap_or("{}".to_owned()))?;
            if try_index > 0 || header["expire"].is_null() {
                header["expire"] = expire.into();
            }
            // parse provided header using calculated value as default for expire param
            let header = Tokenizer::tokenize_optional_params(
                abi.header(),
                &header,
                &HashMap::new())?;
            // take resulting expire value to use it in transaction waiting
            let expire = match header.get("expire").unwrap() {
                TokenValue::Expire(expire) => *expire,
                _ => fail!(SdkError::InternalError{msg: "Wrong expire type".to_owned()})
            };

            Ok((Some(Detokenizer::detokenize_optional(&header)?), Some(expire)))
        } else {
            Ok((header, None))
        }
    }

    // Packs given inputs by abi into Message struct.
    // Works with json representation of input and abi.
    // Returns message's bag of cells and identifier.
    pub fn construct_call_message_json(
        address: MsgAddressInt,
        params: FunctionCallSet,
        internal: bool,
        key_pair: Option<&Keypair>,
        timeouts: Option<&TimeoutsConfig>,
        try_index: Option<u8>,
    ) -> Result<SdkMessage> {
        let (header, expire) = Self::make_expire_header(timeouts, params.abi.clone(), params.header, try_index)?;

        // pack params into bag of cells via ABI
        let msg_body = ton_abi::encode_function_call(
            params.abi, params.func, header, params.input, internal, key_pair,
        )?;

        let msg = Self::create_message(address.clone(), msg_body.into())?;
        let (body, id) = Self::serialize_message(&msg)?;
        Ok(SdkMessage {
            id,
            serialized_message: body,
            message: msg,
            address,
            expire,
        })
    }

    // Packs given inputs by abi into Message struct without sign and returns data to sign.
    // Sign should be then added with `add_sign_to_message` function
    // Works with json representation of input and abi.
    pub fn get_call_message_bytes_for_signing(
        address: MsgAddressInt,
        params: FunctionCallSet,
        timeouts: Option<&TimeoutsConfig>,
        try_index: Option<u8>,
    ) -> Result<MessageToSign> {
        let (header, expire) = Self::make_expire_header(timeouts, params.abi.clone(), params.header, try_index)?;

        // pack params into bag of cells via ABI
        let (msg_body, data_to_sign) = ton_abi::prepare_function_call_for_sign(
            params.abi, params.func, header, params.input,
        )?;

        let msg = Self::create_message(address, msg_body.into())?;

        Self::serialize_message(&msg).map(|(msg_data, _id)| {
            MessageToSign { message: msg_data, data_to_sign, expire }
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
        timeouts: Option<&TimeoutsConfig>,
        try_index: Option<u8>,
    ) -> Result<SdkMessage> {
        let (header, expire) = Self::make_expire_header(timeouts, params.abi.clone(), params.header, try_index)?;

        let msg_body = ton_abi::encode_function_call(
            params.abi, params.func, header, params.input, false, key_pair)?;

        let cell = msg_body.into();
        let msg = Self::create_deploy_message(Some(cell), image, workchain_id)?;

        let address = msg.dst().ok_or_else(|| error!(SdkError::InternalError {
            msg: "No address in created deploy message".to_owned() }))?;
        let (body, id) = Self::serialize_message(&msg)?;

        Ok(SdkMessage {
            id,
            serialized_message: body,
            message: msg,
            address,
            expire,
        })
    }

    // Packs given image and body into Message struct.
    // Returns message's bag of cells and identifier.
    pub fn construct_deploy_message_with_body(image: ContractImage, body: Option<&[u8]>, workchain_id: i32) -> Result<TvmMessage> {
        let body_cell = match body {
            None => None,
            Some(data) => Some(Self::deserialize_tree_to_slice(data)?)
        };

        Self::create_deploy_message(body_cell, image, workchain_id)
    }

    // Packs given image into Message struct.
    // Returns message's bag of cells and identifier.
    pub fn construct_deploy_message_no_constructor(image: ContractImage, workchain_id: i32)
        -> Result<TvmMessage>
    {
        Self::create_deploy_message(None, image, workchain_id)
    }

    // Packs given image and input into Message struct without sign and returns data to sign.
    // Sign should be then added with `add_sign_to_message` function
    // Works with json representation of input and abi.
    pub fn get_deploy_message_bytes_for_signing(
        params: FunctionCallSet,
        image: ContractImage,
        workchain_id: i32,
        timeouts: Option<&TimeoutsConfig>,
        try_index: Option<u8>,
    ) -> Result<MessageToSign> {
        let (header, expire) = Self::make_expire_header(timeouts, params.abi.clone(), params.header, try_index)?;

        let (msg_body, data_to_sign) = ton_abi::prepare_function_call_for_sign(
            params.abi, params.func, header, params.input)?;

        let cell = msg_body.into();
        let msg = Self::create_deploy_message(Some(cell), image, workchain_id)?;

        Self::serialize_message(&msg).map(|(msg_data, _id)| {
            MessageToSign { message: msg_data, data_to_sign, expire }
        })
    }


    // Add sign to message, returned by `get_deploy_message_bytes_for_signing` or
    // `get_run_message_bytes_for_signing` function.
    // Returns serialized message and identifier.
    pub fn add_sign_to_message(
        abi: String,
        signature: &[u8],
        public_key: Option<&[u8]>,
        message: &[u8]
    ) -> Result<SdkMessage> {
        let mut slice = Self::deserialize_tree_to_slice(message)?;

        let mut message: TvmMessage = TvmMessage::construct_from(&mut slice)?;

        let body = message.body()
            .ok_or(error!(SdkError::InvalidData { msg: "No message body".to_owned() }))?;

        let signed_body = ton_abi::add_sign_to_function_call(abi, signature, public_key, body)?;
        message.set_body(signed_body.into());

        let address = message.dst().ok_or_else(|| error!(SdkError::InternalError {
            msg: "No address in signed message".to_owned() }))?;
        let (body, id) = Self::serialize_message(&message)?;

        Ok(SdkMessage {
            id,
            address,
            serialized_message: body,
            message,
            expire: None
        })
    }

    fn create_message(address: MsgAddressInt, msg_body: SliceData) -> Result<TvmMessage> {
        let mut msg_header = ExternalInboundMessageHeader::default();
        msg_header.dst = address;

        let mut msg = TvmMessage::with_ext_in_header(msg_header);
        msg.set_body(msg_body);

        Ok(msg)
    }

    fn create_deploy_message(
        msg_body: Option<SliceData>,
        image: ContractImage,
        workchain_id: i32,
    ) -> Result<TvmMessage> {
        let mut msg_header = ExternalInboundMessageHeader::default();
        msg_header.dst = image.msg_address(workchain_id);
        let mut msg = TvmMessage::with_ext_in_header(msg_header);
        msg.set_state_init(image.state_init());
        msg_body.map(|body| msg.set_body(body));
        Ok(msg)
    }

    pub fn serialize_message(msg: &TvmMessage) -> Result<(Vec<u8>, MessageId)> {
        let cells = msg.write_to_new_cell()?.into();

        let mut data = Vec::new();
        let bag = BagOfCells::with_root(&cells);
        bag.write_to(&mut data, false)?;

        Ok((data, (&cells.repr_hash().as_slice()[..]).into()))
    }

    /// Deserializes tree of cells from byte array into `SliceData`
    fn deserialize_tree_to_slice(data: &[u8]) -> Result<SliceData> {
        let mut response_cells = deserialize_cells_tree(&mut Cursor::new(data))?;

        if response_cells.len() != 1 {
            bail!(SdkError::InvalidData { msg: "Deserialize message error".to_owned() } );
        }

        Ok(response_cells.remove(0).into())
    }

    pub fn get_dst_from_msg(msg: &[u8]) -> Result<MsgAddressInt> {
        let msg = Contract::deserialize_message(msg)?;
    
        msg.dst().ok_or(SdkError::InvalidData {
            msg: "Wrong message type (extOut)".to_owned() }.into())
    }

    /// Deserializes TvmMessage from byte array
    pub fn deserialize_message(message: &[u8]) -> Result<TvmMessage> {
        let mut root_cells = deserialize_cells_tree(&mut Cursor::new(message))?;

        if root_cells.len() != 1 {
            bail!(SdkError::InvalidData { msg: "Deserialize message error".to_owned() } );
        }

        Ok(TvmMessage::construct_from(&mut root_cells.remove(0).into())?)
    }

    fn balance_other_as_hashmape(&self) -> Result<HashmapE> {
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

    pub fn to_account(&self) -> Result<Account> {
        if AccountStatus::AccStateFrozen == self.acc_type {
            return Err(SdkError::InvalidData { msg: "Account is frozen".to_owned()}.into());
        }
        let state = match &self.code {
            Some(code) => {
                let mut state_init = StateInit::default();
                state_init.code = Some(code.clone());
                state_init.data = self.data.clone();
                AccountState::with_state(state_init)
            }
            // account without code is considered uninit
            None => AccountState::AccountUninit
        };
        let storage = AccountStorage {
            last_trans_lt: 0,
            balance: CurrencyCollection { grams: self.balance.into(), other: self.balance_other_as_hashmape()?.into() },
            state,
        };
        let last_paid = if self.last_paid != 0 { self.last_paid } else { Self::now() };
        Ok(Account::with_storage(
            &self.id,
            &StorageInfo::with_values(last_paid, None),
            &storage))
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
        };
        fail!(SdkError::NotFound(format!("No matching shard for account {}", address)))
    }
}


#[cfg(test)]
#[path = "tests/test_contract.rs"]
pub mod tests;
