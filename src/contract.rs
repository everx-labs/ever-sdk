use crate::*;
use std::io::Read;
use std::marker::PhantomData;
use tvm::stack::SliceData;
use tvm::types::AccountId;
use reql::{Client, Connection, Run, Document};
use futures::stream::{Stream, Map};
use futures::future::Future;
use abi_lib::types::{ABIInParameter, ABIOutParameter, ABITypeSignature};

const DB_NAME: &str = "blockchain";
const MSG_TABLE_NAME: &str = "messages";
const MSG_ID_FIELD_NAME: &str = "id";
const MSG_STATE_FIELD_NAME: &str = "state";

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
    code: SliceData,
    data: SliceData,
    lib: SliceData,
}

impl ContractImage {
    fn new(code: &Read, data: Option<&Read>, library: Option<&Read>) -> SdkResult<Self> {
        unimplemented!()
    }

    fn id(&self) -> AccountId {
        unimplemented!()
    }
}

pub struct Contract<TIn: ABIInParameter + ABITypeSignature, TOut: ABIOutParameter + ABITypeSignature> {
    input: PhantomData<TIn>,
    output: PhantomData<TOut>,
    db_connection: Connection,

}

impl<TIn: ABIInParameter + ABITypeSignature, TOut: ABIOutParameter + ABITypeSignature> Contract<TIn, TOut> {

    pub fn load(id: AccountId) -> SdkResult<Box<Future<Item = Contract<TIn, TOut>, Error = SdkError>>> {
        unimplemented!()
    }

    pub fn deploy(image: ContractImage) -> SdkResult<ChangesStream<ContractCallState>> {
        unimplemented!()

        // Deploy is call, but special message is constructed.
        // The message contains StateInit struct with code, public key and lib
        // and body with parameters for contract special method - constructor.

    }

    pub fn call() -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {
        unimplemented!()

        // pack params into bag of cells via ABI
        // message_id = message's hash

        // send message by Kafka
        // (synchroniously - when responce will returned message will be in DB)

        // subscribe on updates from DB and return updates stream

    }

    fn send_message() -> SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {
        unimplemented!()

        // send message by Kafka
    }

    fn subscribe_updates(&self, message_id: MessageId) -> 
        SdkResult<Box<dyn Stream<Item = ContractCallState, Error = SdkError>>> {

        let r = Client::new();

        let map = r.db(DB_NAME)
            .table(MSG_TABLE_NAME)
            .get_all(id_to_string(&message_id))
            .get_field(MSG_STATE_FIELD_NAME)
            .changes()
            .run::<reql_types::Change<MessageState, MessageState>>(self.db_connection)?
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
