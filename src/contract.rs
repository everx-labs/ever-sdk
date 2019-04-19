use crate::*;
use std::io::{Read, Seek};
use std::sync::Arc;
use std::marker::PhantomData;
use tvm::stack::{SliceData, CellData};
use tvm::types::AccountId;
use tvm::cells_serialization::{deserialize_cells_tree, BagOfCells};
use reql::{Client, Connection, Run, Document};
use futures::stream::{Stream, Map};
use futures::future::Future;
use abi_lib::types::{ABIInParameter, ABIOutParameter, ABITypeSignature};
use abi_lib::abi_call::ABICall;
use ed25519_dalek::Keypair;
use rdkafka::producer::future_producer::{FutureProducer, FutureRecord};
use ton_block::{
    Message,
    ExternalInboundMessageHeader,
    MsgAddressInt,
    Serializable,
    Deserializable,
    StateInit,
    GetRepresentationHash};

const DB_NAME: &str = "blockchain";
const MSG_TABLE_NAME: &str = "messages";
const MSG_ID_FIELD_NAME: &str = "id";
const MSG_STATE_FIELD_NAME: &str = "state";
const CONSTRUCTOR_METHOD_NAME: &str = "constructor";
const MESSAGES_TOPIC_NAME: &str = "external messages";

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

impl ContractImage {

    pub fn new<T>(code: &mut T, data: Option<&mut T>, library: Option<&mut T>) -> SdkResult<Self> 
        where T: Read + Seek {

        let mut state_init = StateInit::default();

        let code_roots = deserialize_cells_tree(code)?;
        if code_roots.len() != 1 {
            bail!(SdkErrorKind::InvalidData("Invalid code's bag of cells".into()));
        }
        state_init.set_code(code_roots.remove(0));

        if let Some(data_) = data {
            let data_roots = deserialize_cells_tree(data_)?;
            if data_roots.len() != 1 {
                bail!(SdkErrorKind::InvalidData("Invalid data's bag of cells".into()));
            }
            state_init.set_data(data_roots.remove(0));
        }

        if let Some(library_) = data {
            let library_roots = deserialize_cells_tree(library_)?;
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
    db_connection: Connection,
    kafka_produser: FutureProducer
}

impl Contract {

    pub fn load(id: AccountId) -> SdkResult<Box<Future<Item = Contract, Error = SdkError>>> {
        unimplemented!()
    }

    pub fn deploy<TIn, TOut>(input: TIn, image: ContractImage, key_pair: Option<&Keypair>)
        -> SdkResult<ChangesStream<ContractCallState>>
        where
            TIn: ABIInParameter + ABITypeSignature,
            TOut: ABIInParameter + ABITypeSignature {

        // Deploy is call, but special message is constructed.
        // The message contains StateInit struct with code, public key and lib
        // and body with parameters for contract special method - constructor.

        let msg_body = Self::create_message_body::<TIn, TOut>(input, key_pair);
        let (account_id, msg) = Self::create_deploy_message(msg_body, image)?;
        send_message(msg)
    }

    pub fn call<TIn>(&self, input: TIn, pair: &Keypair)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>>
        where TIn: ABIInParameter + ABITypeSignature {

        unimplemented!()

        // pack params into bag of cells via ABI
        // message_id = message's hash

        // send message by Kafka
        // (synchroniously - when responce will returned message will be in DB)

        // subscribe on updates from DB and return updates stream

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
        -> SdkResult<(AccountId, Message)> {

        let state_init = image.state_init();
        let account_id = state_init.hash()?;

        let mut msg_header = ExternalInboundMessageHeader::default();
        msg_header.dst = MsgAddressInt::with_standart(None, -1, account_id.clone()).unwrap();

        let mut msg = Message::with_ext_in_header(msg_header);
        msg.body = Some(msg_body);
        msg.init = Some(state_init);

        Ok((account_id, msg))
    }

    fn send_message(db_connection: &Connection, kafka_produser: &FutureProducer, msg: Message)
        -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {

        // Prepare

        let cells = msg.write_to_new_cell()?.into();
        let mut data = Vec::new();
        let bag = BagOfCells::with_root(cells);
        let id = bag.get_repr_hash_by_index(0)
            .ok_or(SdkErrorKind::InternalError("unexpected message's bag of cells (empty bag)".into())
                .into())?;                
        bag.write_to(&mut data, false)?;

        // Send by Kafka
        let record = FutureRecord::to(MESSAGES_TOPIC_NAME)
            .key(id.as_slice())
            .payload(&data);

        let id_ = id.clone();
        let chain = kafka_produser.send(record, 0)
            .into_stream()
            .map(|_| {
                ContractCallState {
                    message_id: id_.clone(),
                    message_state: MessageState::Unknown,
                    transaction: None
                }
            }).map_err(|_| SdkErrorKind::Cancelled.into())
            .chain(
                // Subscribe rethink db updates
                Box::leak(Self::subscribe_updates(db_connection, id.clone())?)
            );

            Ok(Box::new(chain))
    }

    fn subscribe_updates(db_connection: &Connection, message_id: MessageId) ->
        SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {

        let r = Client::new();

        let map = r.db(DB_NAME)
            .table(MSG_TABLE_NAME)
            .get_all(id_to_string(&message_id))
            .get_field(MSG_STATE_FIELD_NAME)
            .changes()
            .run::<reql_types::Change<MessageState, MessageState>>(db_connection)?
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
            }).map_err(|err| SdkError::from(err));

        Ok(Box::new(map))
    }
}
