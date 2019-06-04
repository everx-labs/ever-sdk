use crate::*;
use std::io::{Read, Seek};
use std::sync::Arc;
use tvm::stack::{CellData, SliceData, BuilderData};
use tvm::types::AccountId;
use tvm::cells_serialization::{deserialize_cells_tree, BagOfCells};
use reql::Document;
use futures::stream::Stream;
use ton_abi_core::types::{ABIInParameter, ABIOutParameter, ABITypeSignature};
use ton_abi_core::abi_response::ABIResponse;
use ton_abi_core::abi_call::ABICall;
use ton_abi_json::json_abi::encode_function_call;
use ed25519_dalek::{Keypair, PublicKey};
use ton_block::{
    MessageId,    
    ExternalInboundMessageHeader,
    MsgAddressInt,
    Serializable,    
    StateInit,
    GetRepresentationHash,
    Deserializable,
    Grams,
    CurrencyCollection,
    MessageProcessingStatus};
use std::convert::Into;

const MSG_TABLE_NAME: &str = "messages";
const CONTRACTS_TABLE_NAME: &str = "accounts";
const MSG_STATE_FIELD_NAME: &str = "status";
const CONSTRUCTOR_METHOD_NAME: &str = "constructor";

#[cfg(test)]
#[path = "tests/test_contract.rs"]
mod tests;

// The struct represents status of message that performs contract's call
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ContractCallState {
    pub message_id: MessageId,
    pub message_state: MessageProcessingStatus,
}

// The struct represents conract's image
pub struct ContractImage {
    state_init: StateInit,
    id: AccountId
}

#[allow(dead_code)]
impl ContractImage {

    // Creating contract image from code data and library bags of cells
    pub fn new<T>(code: &mut T, data: Option<&mut T>, library: Option<&mut T>) -> SdkResult<Self> 
        where T: Read + Seek {

        let mut state_init = StateInit::default();

        let mut code_roots = deserialize_cells_tree(code)?;
        if code_roots.len() != 1 {
            bail!(SdkErrorKind::InvalidData("Invalid code's bag of cells".into()));
        }
        state_init.set_code(code_roots.remove(0));

        if let Some(data_) = data {
            let mut data_roots = deserialize_cells_tree(data_)?;
            if data_roots.len() != 1 {
                bail!(SdkErrorKind::InvalidData("Invalid data's bag of cells".into()));
            }
            state_init.set_data(data_roots.remove(0));
        }

        if let Some(library_) = library {
            let mut library_roots = deserialize_cells_tree(library_)?;
            if library_roots.len() != 1 {
                bail!(SdkErrorKind::InvalidData("Invalid library's bag of cells".into()));
            }
            state_init.set_library(library_roots.remove(0));
        }

        let id = state_init.hash()?;

        Ok(Self{ state_init, id })
    }

    pub fn from_state_init_and_key<T>(state_init_bag: &mut T, pub_key: &PublicKey) -> SdkResult<Self> 
        where T: Read + Seek {

        let mut si_roots = deserialize_cells_tree(state_init_bag)?;
        if si_roots.len() != 1 {
            bail!(SdkErrorKind::InvalidData("Invalid state init's bag of cells".into()));
        }

        let mut state_init : StateInit
            = StateInit::construct_from(&mut SliceData::from(si_roots.remove(0)))?;

        // state init's data's root cell contains zero-key
        // need to change it by real public key
        let mut new_data: BuilderData;
        if let Some(ref data) = state_init.data {            
            new_data = BuilderData::from(&data); 
            new_data.update_cell(|data, _, _, _, _| {
                let mut vec = Vec::from(&pub_key.as_bytes().clone()[..]); 
                vec.push(0x80);
                *data = vec;
            }, ());
        } else {
            new_data = BuilderData::new();
            new_data.update_cell(|data, _, _, _, _| {
                let mut vec = Vec::from(&pub_key.as_bytes().clone()[..]); 
                vec.push(0x80);
                *data = vec;
            }, ());
        }
        state_init.set_data(Arc::new(new_data.cell().clone()));

        let id = state_init.hash()?;

        Ok(Self{ state_init, id })
    }

    // Returns future contract's state_init struct
    pub fn state_init(self) -> StateInit {
        self.state_init
    }

    // Returns future contract's identifier
    pub fn account_id(&self) -> AccountId {
        self.id.clone()
    }
}

// The struct represents smart contract and allows 
// to deploy and call it, to get some contract's properties.
// Don't forget - in TON blockchain Contract and Account are the same substances.
pub struct Contract {
    acc: ton_block::Account,
}

#[allow(dead_code)]
impl Contract {

    // Asynchronously loads a Contract instance or None if contract with given id is not exists
    pub fn load(id: AccountId) -> SdkResult<Box<Stream<Item = Option<Contract>, Error = SdkError>>> {
        let map = db_helper::load_record(CONTRACTS_TABLE_NAME, &id.to_hex_string())?
            .and_then(|val| {
                if val == serde_json::Value::Null {
                    Ok(None)
                } else {
                    let acc: ton_block::Account = serde_json::from_value(val)
                        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing account: {}", err)))?;

                    Ok(Some(Contract { acc }))
                }
            });

        Ok(Box::new(map))
    }

    // Packs given inputs by abi and asynchronously calls contract.
    // To get calling result - need to load message,
    // it's id and processing status is returned by this function
    pub fn call<TIn, TOut>(id: AccountId, func: String, input: TIn, key_pair: Option<&Keypair>)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>>
        where 
            TIn: ABIInParameter + ABITypeSignature,
            TOut: ABIOutParameter + ABITypeSignature {

        // pack params into bag of cells via ABI
        let msg_body = Self::create_message_body::<TIn, TOut>(func, input, key_pair);
        
        let msg = Self::create_message(id.clone(), msg_body)?;

        // send message by Kafka
        let msg_id = Self::_send_message(msg)?;

        // subscribe on updates from DB and return updates stream
        Self::subscribe_updates(msg_id)
    }

    // Asynchronously calls contract by sending given message.
    // To get calling result - need to load message,
    // it's id and processing status is returned by this function
    pub fn send_message(msg: ton_block::Message)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {

        // send message by Kafka
        let msg_id = Self::_send_message(msg)?;

        // subscribe on updates from DB and return updates stream
        Self::subscribe_updates(msg_id)
    }

    /// Decodes output parameters returned by contract function call 
    pub fn decode_function_response_json(abi: String, function: String, response: Arc<CellData>) 
        -> Result<String, SdkError> {

        ton_abi_json::json_abi::decode_function_response(abi, function, SliceData::from(response))
            .map_err(|err| SdkError::from(SdkErrorKind::AbiError(err)))
    }
    
    /// Decodes ABI contract answer from `Vec<u8>` into type values
    pub fn decode_function_response<TOut>(response: Arc<CellData>)
        -> Result<TOut::Out, SdkError> 
        where TOut: ABIOutParameter + ABITypeSignature {

        ABIResponse::<TOut>::decode_response_from_slice(SliceData::from(response))
            .map_err(|err| SdkError::from(SdkErrorKind::AbiError2(err)))
    }

    // Packs given inputs by abi into ton_block::Message struct.
    // Returns message's bag of cells and identifier.
    pub fn construct_call_message<TIn, TOut>(id: AccountId, func: String, input: TIn, key_pair: Option<&Keypair>)
        -> SdkResult<(Vec<u8>, MessageId)>
        where 
            TIn: ABIInParameter + ABITypeSignature,
            TOut: ABIOutParameter + ABITypeSignature {

        // pack params into bag of cells via ABI
        let msg_body = Self::create_message_body::<TIn, TOut>(func, input, key_pair);
        
        let msg = Self::create_message(id.clone(), msg_body)?;

        Self::serialize_message(msg)
    }

    // Packs given inputs by abi and asynchronously calls contract.
    // Works with json representation of input and abi.
    // To get calling result - need to load message,
    // it's id and processing status is returned by this function
    pub fn call_json(id: AccountId, func: String, input: String, abi: String, key_pair: Option<&Keypair>)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {

        // pack params into bag of cells via ABI
        let msg_body = encode_function_call(abi, func, input, key_pair)
            .map_err(|err| SdkError::from(SdkErrorKind::AbiError(err)))?;
        
        let msg = Self::create_message(id.clone(), msg_body.into())?;

        // send message by Kafka
        let msg_id = Self::_send_message(msg)?;

        // subscribe on updates from DB and return updates stream
        Self::subscribe_updates(msg_id)
    }

    // Asynchronously loads a Message's json representation 
    // or null if message with given id is not exists
    pub fn load_json(id: AccountId) -> SdkResult<Box<Stream<Item = String, Error = SdkError>>> {

        let map = db_helper::load_record(CONTRACTS_TABLE_NAME, &id.to_hex_string())?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    // Packs given image and input and asynchronously calls contract's constructor.
    // To get deploying result - need to load message,
    // it's id and processing status is returned by this function
    pub fn deploy<TIn, TOut>(input: TIn, image: ContractImage, key_pair: Option<&Keypair>)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>>
        where
            TIn: ABIInParameter + ABITypeSignature,
            TOut: ABIOutParameter + ABITypeSignature {

        // Deploy is call, but special message is constructed.
        // The message contains StateInit struct with code, public key and lib
        // and body with parameters for contract special method - constructor.

        let msg_body = Self::create_message_body::<TIn, TOut>(CONSTRUCTOR_METHOD_NAME.to_string(), input, key_pair);

        let msg = Self::create_deploy_message(Some(msg_body), image)?;

        let msg_id = Self::_send_message(msg)?;

        Self::subscribe_updates(msg_id)
    }

    // Packs given image and input into ton_block::Message struct.
    // Returns message's bag of cells and identifier.
    pub fn construct_deploy_message<TIn, TOut>(input: TIn, image: ContractImage, key_pair: Option<&Keypair>)
        -> SdkResult<(Vec<u8>, MessageId)>
        where
            TIn: ABIInParameter + ABITypeSignature,
            TOut: ABIOutParameter + ABITypeSignature {

        let msg_body = Self::create_message_body::<TIn, TOut>(CONSTRUCTOR_METHOD_NAME.to_string(), input, key_pair);

        let msg = Self::create_deploy_message(Some(msg_body), image)?;

        Self::serialize_message(msg)
    }

    // Returns contract's identifier
    pub fn id(&self) -> AccountId {
        self.acc.get_id().unwrap().clone()
    }

    // Returns contract's balance in NANO grams
    pub fn balance_grams(&self) -> Grams {
        self.acc.get_balance().unwrap().grams.clone()
    }

    // Returns contract's balance 
    pub fn balance(&self) -> CurrencyCollection {
        unimplemented!()
    }

    // Returns blockchain's account struct
    // Some node-specifed methods won't work. All TonStructVariant fields has Client variant.
    pub fn acc(&self) -> &ton_block::Account {
         &self.acc
    }

    // Packs given image and input and asynchronously calls given contract's constructor method.
    // Works with json representation of input and abi.
    // To get calling result - need to load message,
    // it's id and processing status is returned by this function
    pub fn deploy_json(func: String, input: String, abi: String, image: ContractImage, key_pair: Option<&Keypair>)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {

        let msg_body = encode_function_call(abi, func, input, key_pair)
            .map_err(|err| SdkError::from(SdkErrorKind::AbiError(err)))?;

        let cell: std::sync::Arc<tvm::stack::CellData> = msg_body.into();
        let msg = Self::create_deploy_message(Some(cell), image)?;

        let msg_id = Self::_send_message(msg)?;

        Self::subscribe_updates(msg_id)
    }

    // Packs given image asynchronously send deploy message into blockchain.    
    // To get calling result - need to load message,
    // it's id and processing status is returned by this function
    pub fn deploy_no_constructor(image: ContractImage)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {
        let msg = Self::create_deploy_message(None, image)?;

        let msg_id = Self::_send_message(msg)?;

        Self::subscribe_updates(msg_id)
    }

    fn create_message(id: AccountId, msg_body: Arc<CellData>)
        -> SdkResult<ton_block::Message> {

        let mut msg_header = ExternalInboundMessageHeader::default();
        msg_header.dst = MsgAddressInt::with_standart(None, -1, id).unwrap();

        let mut msg = ton_block::Message::with_ext_in_header(msg_header);
        msg.body = Some(msg_body);        

        Ok(msg)
    }

    fn create_message_body<TIn, TOut>(func: String, input: TIn, key_pair: Option<&Keypair>) -> Arc<CellData>
        where
            TIn: ABIInParameter + ABITypeSignature,
            TOut: ABIOutParameter + ABITypeSignature {

        match key_pair {
            Some(p) => {
                ABICall::<TIn, TOut>::encode_signed_function_call_into_slice(
                    func, input, p).into()
            }
            _ => {
                ABICall::<TIn, TOut>::encode_function_call_into_slice(
                    func, input).into()
            }
        }
    }

    fn create_deploy_message(msg_body: Option<Arc<CellData>>, image: ContractImage)
        -> SdkResult<ton_block::Message> {

        let account_id = image.account_id();
        let state_init = image.state_init();

        let mut msg_header = ExternalInboundMessageHeader::default();
        msg_header.dst = MsgAddressInt::with_standart(None, -1, account_id).unwrap();

        let mut msg = ton_block::Message::with_ext_in_header(msg_header);
        msg.body = msg_body;
        msg.init = Some(state_init);

        Ok(msg)
    }

    fn _send_message(msg: ton_block::Message) -> SdkResult<MessageId> {
        let (data, id) = Self::serialize_message(msg)?;
       
        kafka_helper::send_message(&id.as_slice()[..], &data)?;
        println!("msg is sent, id: {}", id.to_hex_string());
        Ok(id.clone())
    }

    fn serialize_message(msg: ton_block::Message) -> SdkResult<(Vec<u8>, MessageId)> {
        let cells = msg.write_to_new_cell()?.into();
        let mut data = Vec::new();
        let bag = BagOfCells::with_root(cells);
        let id = bag.get_repr_hash_by_index(0)
            .ok_or::<SdkError>(SdkErrorKind::InternalError("unexpected message's bag of cells (empty bag)".into())
                .into())?.clone();
        bag.write_to(&mut data, false)?;

        Ok((data, id))
    }

    fn subscribe_updates(message_id: MessageId) ->
        SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {

        let map = db_helper::subscribe_field_updates(
                MSG_TABLE_NAME,
                &message_id.to_hex_string(),
                MSG_STATE_FIELD_NAME
            )?
            .map(move |change_opt| {
                match change_opt {
                    Some(Document::Expected(state_change)) => {
                        ContractCallState {
                            message_id: message_id.clone(),
                            message_state: state_change.new_val.unwrap_or_else(|| MessageProcessingStatus::Unknown),
                        }
                    },
                    _ => {
                        ContractCallState {
                            message_id: message_id.clone(),
                            message_state: MessageProcessingStatus::Unknown,
                        }
                    },
                }
            });

        Ok(Box::new(map))
    }
}
