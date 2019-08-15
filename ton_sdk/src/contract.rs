use crate::*;
use std::io::{Read, Seek, Cursor};
use std::sync::{Arc, Mutex};
use ed25519_dalek::{Keypair, PublicKey};
use tvm::stack::{BuilderData, CellData, IBitstring, SliceData};
use tvm::types::AccountId;
use tvm::cells_serialization::{deserialize_cells_tree, BagOfCells};
use ton_abi_core::types::{ABIInParameter, ABIOutParameter, ABITypeSignature};
use ton_abi_core::abi_response::ABIResponse;
use ton_abi_core::abi_call::ABICall;
use tvm::block::{
    Account,
    Message as TvmMessage, 
    MessageId,
    ExternalInboundMessageHeader,
    MsgAddressExt, 
    MsgAddressInt,
    Serializable,
    StateInit,
    GetRepresentationHash,
    Deserializable,
    Grams,
    CurrencyCollection,
    MessageProcessingStatus
};
use std::convert::Into;

#[cfg(feature = "node_interaction")]
use futures::stream::Stream;

const CONSTRUCTOR_METHOD_NAME: &str = "constructor";

lazy_static! {
    static ref DEFAULT_WORKCHAIN: Mutex<Option<i32>> = Mutex::new(None);
}

#[cfg(test)]
#[path = "tests/test_contract.rs"]
mod tests;

// The struct represents status of message that performs contract's call
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ContractCallState {
    pub id: MessageId,
    pub status: MessageProcessingStatus,
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

        let id = state_init.hash()?.into();

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
        state_init.set_data(new_data.into());

        let id = state_init.hash()?.into();

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
    acc: Account,
}

/// Enum represents blockchain account address.
/// `Short` value contains only `AccountId` value and is used for addressing contracts in default
/// workchain. `Full` value is fully qualified account address and can be used for addressing
/// contracts in any workchain
pub enum AccountAddress {
    Short(AccountId),
    Full(MsgAddressInt)
}

impl AccountAddress {
    /// Returns `AccountId` from the address
    pub fn get_account_id(&self) -> SdkResult<AccountId> {
        match self {
            AccountAddress::Short(account_id) => Ok(account_id.clone()),
            AccountAddress::Full(address) => {
                let vec = address.get_address();
                if vec.remaining_bits() == 256 {
                    Ok(vec)
                } else {
                    Err(SdkErrorKind::InvalidData("Address must be 32 bytes long".to_owned()).into())
                }
            }
        }
    }

    /// Returns full account address as `MsgAddressInt` struct
    pub fn get_msg_address(&self) -> SdkResult<MsgAddressInt> {
        match self {
            AccountAddress::Full(address) => Ok(address.clone()),
            AccountAddress::Short(id) => {
                let workchain = Contract::get_default_workchain()
                    .ok_or(SdkErrorKind::DefaultWorkchainNotSet)?;

                Ok(MsgAddressInt::with_standart(None, workchain as i8, id.clone())?)
            }
        }
    }
}

impl From<AccountId> for AccountAddress {
    fn from(data: AccountId) -> Self {
        AccountAddress::Short(data)
    }
}

impl From<MsgAddressInt> for AccountAddress {
    fn from(data: MsgAddressInt) -> Self {
        AccountAddress::Full(data)
    }
}

#[allow(dead_code)]
#[cfg(feature = "node_interaction")]
impl Contract {

    // Asynchronously loads a Contract instance or None if contract with given id is not exists
    pub fn load(address: AccountAddress) -> SdkResult<Box<Stream<Item = Option<Contract>, Error = SdkError>>> {
        let id = address.get_account_id()?;

        let map = db_helper::load_record(CONTRACTS_TABLE_NAME, &id.to_hex_string())?
            .and_then(|val| {
                if val == serde_json::Value::Null {
                    Ok(None)
                } else {
                    let acc: Account = serde_json::from_value(val)
                        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing account: {}", err)))?;

                    Ok(Some(Contract { acc }))
                }
            });

        Ok(Box::new(map))
    }

    // Asynchronously loads a Message's json representation
    // or null if message with given id is not exists
    pub fn load_json(id: AccountId) -> SdkResult<Box<Stream<Item = String, Error = SdkError>>> {

        let map = db_helper::load_record(CONTRACTS_TABLE_NAME, &id.to_hex_string())?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    // Packs given inputs by abi and asynchronously calls contract.
    // To get calling result - need to load message,
    // it's id and processing status is returned by this function
    pub fn call<TIn, TOut>(address: AccountAddress, func: String, input: TIn, key_pair: Option<&Keypair>)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>>
        where
            TIn: ABIInParameter + ABITypeSignature,
            TOut: ABIOutParameter + ABITypeSignature {

        // pack params into bag of cells via ABI
        let msg_body = Self::create_message_body::<TIn, TOut>(func, input, key_pair);

        let msg = Self::create_message(address, msg_body)?;

        // send message by Kafka
        let msg_id = Self::_send_message(msg)?;

        // subscribe on updates from DB and return updates stream
        Self::subscribe_updates(msg_id)
    }

    // Packs given inputs by abi and asynchronously calls contract.
    // Works with json representation of input and abi.
    // To get calling result - need to load message,
    // it's id and processing status is returned by this function
    pub fn call_json(id: AccountId, func: String, input: String, abi: String, key_pair: Option<&Keypair>)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {

        // pack params into bag of cells via ABI
        let msg_body = ton_abi_json::encode_function_call(abi, func, input, key_pair)
            .map_err(|err| SdkError::from(SdkErrorKind::AbiError(err)))?;

        let msg = Self::create_message(id.clone().into(), msg_body.into())?;

        // send message by Kafka
        let msg_id = Self::_send_message(msg)?;

        // subscribe on updates from DB and return updates stream
        Self::subscribe_updates(msg_id)
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

    // Packs given image and input and asynchronously calls given contract's constructor method.
    // Works with json representation of input and abi.
    // To get calling result - need to load message,
    // it's id and processing status is returned by this function
    pub fn deploy_json(func: String, input: String, abi: String, image: ContractImage, key_pair: Option<&Keypair>)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {

        let msg_body = ton_abi_json::encode_function_call(abi, func, input, key_pair)
            .map_err(|err| SdkError::from(SdkErrorKind::AbiError(err)))?;

        let cell = msg_body.into();
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

    // Asynchronously calls contract by sending given message.
    // To get calling result - need to load message,
    // it's id and processing status is returned by this function
    pub fn send_message(msg: TvmMessage)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> 
    {
        // send message by Kafka
        let msg_id = Self::_send_message(msg)?;
        // subscribe on updates from DB and return updates stream
        Self::subscribe_updates(msg_id)
    }

    fn _send_message(msg: TvmMessage) -> SdkResult<MessageId> {
        let (data, id) = Self::serialize_message(msg)?;

        kafka_helper::send_message(&id.as_slice()[..], &data)?;
        println!("msg is sent, id: {}", id.to_hex_string());
        Ok(id.clone())
    }

    pub fn send_serialized_message(id: MessageId, msg: &[u8]) -> SdkResult<()> {
        kafka_helper::send_message(&id.as_slice()[..], msg)
    }

    pub fn subscribe_updates(message_id: MessageId) ->
        SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {

        /*let load_stream = Message::load(message_id.clone())?
            .filter_map(|msg_option| {
                msg_option.map(|msg| {
                    ContractCallState {message_id: msg.id(), status: msg.status()}
                })
            });*/

        let load_stream = db_helper::load_record_fields(
            MESSAGES_TABLE_NAME,
            &message_id.to_hex_string(), 
            CONTRACT_CALL_STATE_FIELDS)?;

        let subscribe_stream = db_helper::subscribe_record_updates(
            MESSAGES_TABLE_NAME,
            &message_id.to_hex_string(), 
            CONTRACT_CALL_STATE_FIELDS)?;

        let subscribe_stream = load_stream.chain(subscribe_stream);

        let subscribe_stream = subscribe_stream
            .then(|result| {
                match result {
                    Err(err) => Err(SdkError::from(err)),
                    Ok(value) => {
                        println!("{}", value);
                        Ok(serde_json::from_value::<ContractCallState>(value)?)
                    }
                }
        });

        Ok(Box::new(subscribe_stream))
        //Ok(Box::new(load_stream.chain(subscribe_stream)))
    }
}

pub struct MessageToSign {
    pub message: Vec<u8>,
    pub data_to_sign: Vec<u8>
}

impl Contract {

    /// Decodes output parameters returned by contract function call 
    pub fn decode_function_response_json(abi: String, function: String, response: SliceData) 
        -> SdkResult<String> {

        ton_abi_json::json_abi::decode_function_response(abi, function, response)
            .map_err(|err| SdkError::from(SdkErrorKind::AbiError(err)))
    }

    /// Decodes ABI contract answer from `CellData` into type values
    pub fn decode_function_response<TOut>(response: SliceData)
        -> SdkResult<TOut::Out>
        where TOut: ABIOutParameter{

        ABIResponse::<TOut>::decode_response_from_slice(response)
            .map_err(|err| SdkError::from(SdkErrorKind::AbiError2(err)))
    }

    /// Decodes output parameters returned by contract function call from serialized message body
    pub fn decode_function_response_from_bytes_json(abi: String, function: String, response: &[u8])
        -> SdkResult<String> {

        let mut response_cells = deserialize_cells_tree(&mut Cursor::new(response))?;

        if response_cells.len() != 1 {
            return Err(SdkError::from(SdkErrorKind::InvalidData("Deserialize message error".to_owned())));
        }

        Self::decode_function_response_json(abi, function, response_cells.remove(0).into())
    }

    /// Decodes output parameters returned by contract function call from serialized message body
    pub fn decode_function_response_from_bytes<TOut>(response: &[u8])
         -> SdkResult<TOut::Out>
        where TOut: ABIOutParameter {

        ABIResponse::<TOut>::decode_response(&response.to_vec())
            .map_err(|err| SdkError::from(SdkErrorKind::AbiError2(err)))
    }

    // Packs given inputs by abi into Message struct.
    // Returns message's bag of cells and identifier.
    pub fn construct_call_message<TIn, TOut>(address: AccountAddress, func: String, input: TIn, key_pair: Option<&Keypair>)
        -> SdkResult<(Vec<u8>, MessageId)>
        where
            TIn: ABIInParameter + ABITypeSignature,
            TOut: ABIOutParameter + ABITypeSignature {

        // pack params into bag of cells via ABI
        let msg_body = Self::create_message_body::<TIn, TOut>(func, input, key_pair);

        let msg = Self::create_message(address, msg_body)?;

        Self::serialize_message(msg)
    }

    // Packs given inputs by abi into Message struct.
    // Works with json representation of input and abi.
    // Returns message's bag of cells and identifier.
    pub fn construct_call_message_json(address: AccountAddress, func: String, input: String,
        abi: String, key_pair: Option<&Keypair>) -> SdkResult<(Vec<u8>, MessageId)> {

        // pack params into bag of cells via ABI
        let msg_body = ton_abi_json::encode_function_call(abi, func, input, key_pair)
            .map_err(|err| SdkError::from(SdkErrorKind::AbiError(err)))?;

        let msg = Self::create_message(address, msg_body.into())?;

        Self::serialize_message(msg)
    }

    // Packs given inputs by abi into Message struct without sign and returns data to sign.
    // Sign should be then added with `add_sign_to_message` function
    // Works with json representation of input and abi.
    pub fn get_call_message_bytes_for_signing(address: AccountAddress, func: String, input: String, 
        abi: String) -> SdkResult<MessageToSign> {
        
        // pack params into bag of cells via ABI
        let (msg_body, data_to_sign) = ton_abi_json::prepare_function_call_for_sign(abi, func, input)
            .map_err(|err| SdkError::from(SdkErrorKind::AbiError(err)))?;

        let msg = Self::create_message(address, msg_body.into())?;

        Self::serialize_message(msg).map(|(msg_data, _id)| {
                MessageToSign { message: msg_data, data_to_sign } 
            }
        )
    }

    // Packs given image and input into Message struct.
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

    // Packs given image and input into Message struct.
    // Works with json representation of input and abi.
    // Returns message's bag of cells and identifier.
    pub fn construct_deploy_message_json(func: String, input: String, abi: String, image: ContractImage,
        key_pair: Option<&Keypair>) -> SdkResult<(Vec<u8>, MessageId)> {

        let msg_body = ton_abi_json::encode_function_call(abi, func, input, key_pair)
            .map_err(|err| SdkError::from(SdkErrorKind::AbiError(err)))?;

        let cell = msg_body.into();
        let msg = Self::create_deploy_message(Some(cell), image)?;

        Self::serialize_message(msg)
    }
    
    // Packs given image and input into Message struct without sign and returns data to sign.
    // Sign should be then added with `add_sign_to_message` function
    // Works with json representation of input and abi.
    pub fn get_deploy_message_bytes_for_signing(func: String, input: String, abi: String,
        image: ContractImage) -> SdkResult<MessageToSign> {

        let (msg_body, data_to_sign) = ton_abi_json::prepare_function_call_for_sign(abi, func, input)
                .map_err(|err| SdkError::from(SdkErrorKind::AbiError(err)))?;

        let cell = msg_body.into();
        let msg = Self::create_deploy_message(Some(cell), image)?;

        Self::serialize_message(msg).map(|(msg_data, _id)| {
                MessageToSign { message: msg_data, data_to_sign } 
            }
        )
    }

    // Add sign to message, returned by `get_deploy_message_bytes_for_signing` or 
    // `get_run_message_bytes_for_signing` function.
    // Returns serialized message and identifier.
    pub fn add_sign_to_message(signature: &[u8], public_key: &[u8], message: &[u8]) 
        -> SdkResult<(Vec<u8>, MessageId)> {
        
        let mut root_cells = deserialize_cells_tree(&mut Cursor::new(message))?;

        if root_cells.len() != 1 { 
            return Err(SdkError::from(SdkErrorKind::InvalidData("Deserialize message error".to_owned())));
        }

        let mut message: TvmMessage = TvmMessage::construct_from(&mut root_cells.remove(0).into())?;

        let body = message.body()
            .ok_or(SdkError::from(SdkErrorKind::InvalidData("No message body".to_owned())))?;

        let signed_body = ton_abi_json::add_sign_to_function_call(signature, public_key, body)
            .map_err(|err| SdkError::from(SdkErrorKind::AbiError(err)))?;

        *message.body_mut() = Some(signed_body.into());
            

        Self::serialize_message(message)
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
    pub fn acc(&self) -> &Account {
         &self.acc
    }

    fn create_message(address: AccountAddress, msg_body: SliceData) -> SdkResult<TvmMessage> {

        let mut msg_header = ExternalInboundMessageHeader::default();
        msg_header.dst = address.get_msg_address()?;


        // TODO don't forget to delete it
        // This is temporary code to make all messages uniq.
        // In the future it will be made by replay attack protection mechanism
        let mut builder = BuilderData::new();
        if cfg!(target_arch="wasm32") {
            use rand::Rng;            
            let mut rng = rand::thread_rng();
            builder.append_u64(rng.gen::<u64>()).unwrap();
        } else {
            use sha2::Digest;
            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap()
                .as_nanos();
            let mut hasher = sha2::Sha256::new();
            hasher.input(&time.to_be_bytes()[..]);
            let hash = hasher.result();
            builder.append_raw(&hash.to_vec()[..], 64).unwrap();
        }
        msg_header.src = MsgAddressExt::with_extern(builder.into()).unwrap(); 
        
        let mut msg = TvmMessage::with_ext_in_header(msg_header);
        *msg.body_mut() = Some(msg_body);

        Ok(msg)
    }

    fn create_message_body<TIn, TOut>(func: String, input: TIn, key_pair: Option<&Keypair>) -> SliceData
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

    fn create_deploy_message(
        msg_body: Option<SliceData>,
        image: ContractImage
    ) -> SdkResult<TvmMessage> {
        let account_id = image.account_id();
        let state_init = image.state_init();
        let mut msg_header = ExternalInboundMessageHeader::default();
        msg_header.dst = AccountAddress::from(account_id).get_msg_address()?;
        let mut msg = TvmMessage::with_ext_in_header(msg_header);
        *msg.body_mut() = msg_body;
        *msg.state_init_mut() = Some(state_init);
        Ok(msg)
    }

    fn serialize_message(msg: TvmMessage) -> SdkResult<(Vec<u8>, MessageId)> {
        let cells = Arc::<CellData>::from(msg.write_to_new_cell()?);
        let id = cells.repr_hash();

        let mut data = Vec::new();
        let bag = BagOfCells::with_root(&cells);
        bag.write_to(&mut data, false)?;

        Ok((data, id.into()))
    }

    /// Sets new default workchain number which will be used in message destination address
    /// construction if client provides only account ID
    pub fn set_default_workchain(workchain: Option<i32>) {
        let mut default = DEFAULT_WORKCHAIN.lock().unwrap();
        *default = workchain;
    }

    /// Returns default workchain number which are used in message destination address
    /// construction if client provides only account ID
    pub fn get_default_workchain() -> Option<i32> {
        let default = DEFAULT_WORKCHAIN.lock().unwrap();

        default.clone()
    }
}
