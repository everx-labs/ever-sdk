use crate::*;
use std::io::{Read, Seek};
use std::sync::Arc;
use tvm::stack::CellData;
use tvm::types::AccountId;
use tvm::cells_serialization::{deserialize_cells_tree, BagOfCells};
use reql::Document;
use futures::stream::Stream;
use abi_lib::types::{ABIInParameter, ABITypeSignature};
use abi_lib::abi_call::ABICall;
use ed25519_dalek::Keypair;
use ton_block::{
    Message,
    ExternalInboundMessageHeader,
    MsgAddressInt,
    Serializable,    
    StateInit,
    GetRepresentationHash};

const MSG_TABLE_NAME: &str = "messages";
const CONTRACTS_TABLE_NAME: &str = "contracts";
const MSG_STATE_FIELD_NAME: &str = "state";
const CONSTRUCTOR_METHOD_NAME: &str = "constructor";

#[cfg(test)]
#[path = "tests/test_contract.rs"]
mod tests;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ContractCallState {
    message_id: MessageId,
    message_state: MessageState,

    // Exists with MessageState::Proposed and MessageState::Finalized
    transaction: Option<TransactionId>
}

pub struct ContractImage {
    state_init: StateInit
}

#[allow(dead_code)]
impl ContractImage {

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
            state_init.set_data(library_roots.remove(0));
        }

        Ok(Self{ state_init })
    }

    pub fn state_init(self) -> StateInit {
        self.state_init
    }
}

pub struct Contract {
    id: AccountId,

}

#[allow(dead_code)]
impl Contract {

    pub fn load(id: AccountId) -> SdkResult<Box<Stream<Item = Contract, Error = SdkError>>> {
        let map = db_helper::load_record(CONTRACTS_TABLE_NAME, &id_to_string(&id))?
            .map(move |_val| Contract { id: id.clone() }); // TODO parse json

        Ok(Box::new(map))
    }

    pub fn call<TIn, TOut>(&self, input: TIn, key_pair: Option<&Keypair>)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>>
        where 
            TIn: ABIInParameter + ABITypeSignature,
            TOut: ABIInParameter + ABITypeSignature {

        // pack params into bag of cells via ABI
        let msg_body = Self::create_message_body::<TIn, TOut>(input, key_pair);
        
        let msg = Self::create_message(self, msg_body)?;

        // send message by Kafka
        let msg_id = Self::send_message(msg)?;

        // subscribe on updates from DB and return updates stream
        Self::subscribe_updates(msg_id)
    }

    pub fn call_json(&self, _func: String, _input: String, _abi: String, _key_pair: Option<&Keypair>)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {

        // pack params into bag of cells via ABI
        let msg_body = Arc::new(CellData::default()); // TODO
        
        let msg = Self::create_message(self, msg_body)?;

        // send message by Kafka
        let msg_id = Self::send_message(msg)?;

        // subscribe on updates from DB and return updates stream
        Self::subscribe_updates(msg_id)
    }

    pub fn load_json(id: AccountId) -> SdkResult<Box<Stream<Item = String, Error = SdkError>>> {

        let map = db_helper::load_record(CONTRACTS_TABLE_NAME, &id_to_string(&id))?
            .map(|val| val.to_string());

        Ok(Box::new(map))
    }

    pub fn deploy<TIn, TOut>(input: TIn, image: ContractImage, key_pair: Option<&Keypair>)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>>
        where
            TIn: ABIInParameter + ABITypeSignature,
            TOut: ABIInParameter + ABITypeSignature {

        // Deploy is call, but special message is constructed.
        // The message contains StateInit struct with code, public key and lib
        // and body with parameters for contract special method - constructor.

        let msg_body = Self::create_message_body::<TIn, TOut>(input, key_pair);

        let msg = Self::create_deploy_message(msg_body, image)?;

        let msg_id = Self::send_message(msg)?;

        Self::subscribe_updates(msg_id)
    }

    pub fn deploy_json(_func: String, _input: String, _abi: String, image: ContractImage, _key_pair: Option<&Keypair>)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {

        let msg_body = Arc::new(CellData::default()); // TODO

        let msg = Self::create_deploy_message(msg_body, image)?;

        let msg_id = Self::send_message(msg)?;

        Self::subscribe_updates(msg_id)
    }

    fn create_message(&self, msg_body: Arc<CellData>)
        -> SdkResult<Message> {

        let mut msg_header = ExternalInboundMessageHeader::default();
        msg_header.dst = MsgAddressInt::with_standart(None, -1, self.id.clone()).unwrap();

        let mut msg = Message::with_ext_in_header(msg_header);
        msg.body = Some(msg_body);        

        Ok(msg)
    }

    fn create_message_body<TIn, TOut>(input: TIn, key_pair: Option<&Keypair>) -> Arc<CellData>
        where
            TIn: ABIInParameter + ABITypeSignature,
            TOut: ABIInParameter + ABITypeSignature {

        match key_pair {
            Some(p) => {
                ABICall::<TIn, TOut>::encode_signed_function_call_into_slice(
                    CONSTRUCTOR_METHOD_NAME, input, p).into()
            }
            _ => {
                ABICall::<TIn, TOut>::encode_function_call_into_slice(
                    CONSTRUCTOR_METHOD_NAME, input).into()
            }
        }
    }

    fn create_deploy_message(msg_body: Arc<CellData>, image: ContractImage)
        -> SdkResult<Message> {

        let state_init = image.state_init();
        let account_id = state_init.hash()?;

        let mut msg_header = ExternalInboundMessageHeader::default();
        msg_header.dst = MsgAddressInt::with_standart(None, -1, account_id).unwrap();

        let mut msg = Message::with_ext_in_header(msg_header);
        msg.body = Some(msg_body);
        msg.init = Some(state_init);

        Ok(msg)
    }

    fn send_message(msg: Message) -> SdkResult<MessageId> {

        let cells = msg.write_to_new_cell()?.into();
        let mut data = Vec::new();
        let bag = BagOfCells::with_root(cells);
        let id = bag.get_repr_hash_by_index(0)
            .ok_or::<SdkError>(SdkErrorKind::InternalError("unexpected message's bag of cells (empty bag)".into())
                .into())?;
        bag.write_to(&mut data, false)?;

        kafka_helper::send_message(&id.as_slice()[..], &data)?;
        println!("msg sent");
        Ok(id.clone())
    }

    fn subscribe_updates(message_id: MessageId) ->
        SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {

        let map = db_helper::subscribe_field_updates(
                MSG_TABLE_NAME,
                &id_to_string(&message_id),
                MSG_STATE_FIELD_NAME
            )?
            .map(move |change_opt| {
                match change_opt {
                    Some(Document::Expected(state_change)) => {

                        // TODO get full message to extract transaction id from

                        ContractCallState {
                            message_id: message_id.clone(),
                            message_state: state_change.new_val.unwrap_or_else(|| MessageState::Unknown),
                            transaction: None
                        }
                    },
                    _ => {
                        ContractCallState {
                            message_id: message_id.clone(),
                            message_state: MessageState::Unknown,
                            transaction: None
                        }
                    },
                }
            });

        Ok(Box::new(map))
    }
}
