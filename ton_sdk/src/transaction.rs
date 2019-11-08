use crate::*;
use futures::stream::Stream;
use tvm::block::{TransactionProcessingStatus, AccStatusChange, ComputeSkipReason};
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct ComputePhase {
    pub compute_type: u8,
    #[serde(deserialize_with = "json_helper::deserialize_skipped_reason")]
    pub skipped_reason: Option<ComputeSkipReason>,
    pub exit_code: i32,
    pub success: bool
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct StoragePhase {
    #[serde(deserialize_with = "json_helper::deserialize_acc_state_change")]
    pub status_change: AccStatusChange
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct ActionPhase {
    pub success: bool,
    pub valid: bool,
    pub no_funds: bool,
    pub result_code: i32
}

pub type TransactionId = StringId;

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct Transaction {
    pub id: TransactionId,
    #[serde(deserialize_with = "json_helper::deserialize_tr_state")]
    pub status: TransactionProcessingStatus,
    pub now: u32,
    pub in_msg: Option<MessageId>,
    pub out_msgs: Vec<MessageId>,
    pub aborted: bool,
    pub compute: ComputePhase,
    pub storage: Option<StoragePhase>,
    pub action: ActionPhase
}

// The struct represents performed transaction and allows to access their properties.
#[allow(dead_code)]
impl Transaction {

    // Asynchronously loads a Transaction instance or None if transaction with given id is not exists
    pub fn load(id: &TransactionId) -> SdkResult<Box<dyn Stream<Item = Option<Transaction>, Error = SdkError>>> {
        let map = queries_helper::load_record_fields(
            TRANSACTIONS_TABLE_NAME,
            &id.to_string(),
            TRANSACTION_FIELDS_ORDINARY)?
                .and_then(|val| {
                    if val == serde_json::Value::Null {
                        Ok(None)
                    } else {
                        let tr: Transaction = serde_json::from_value(val)
                            .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing transaction: {}", err)))?;

                        Ok(Some(tr))
                    }
            });

        Ok(Box::new(map))
    }

    // Returns transaction's processing status
    pub fn status(&self) -> TransactionProcessingStatus {
        self.status
    }

    // Returns id of transaction's input message if it exists
    pub fn in_message_id(&self) -> Option<MessageId> {
        self.in_msg.clone()
    }

    // Asynchronously loads an instance of transaction's input message
    pub fn load_in_message(&self) -> SdkResult<Box<dyn Stream<Item = Option<Message>, Error = SdkError>>> {
        match self.in_message_id() {
            Some(m) => Message::load(&m),
            None => bail!(SdkErrorKind::InvalidOperation("transaction doesn't have inbound message".into()))
        }
    }

    // Returns id of transaction's out messages if it exists
    pub fn out_messages_id(&self) -> &Vec<MessageId> {
        &self.out_msgs
    }

    // Returns message's identifier
    pub fn id(&self) -> TransactionId {
        // On client side id is ready allways. It is never be calculated, just returned.
        self.id.clone()
    }

    // Returns `aborted` flag
    pub fn is_aborted(&self) -> bool {
        self.aborted
    }

    // Asynchronously loads an instances of transaction's out messages
    pub fn load_out_messages(&self) -> SdkResult<Box<dyn Stream<Item = Option<Message>, Error = SdkError>>> {
        let mut msg_id_iter = self.out_messages_id().iter();
        if let Some(id) = msg_id_iter.next().clone() {
            let mut stream = Message::load(&id)?;
            for id in msg_id_iter {
                stream = Box::new(stream.chain(Message::load(&id)?));
            }
            Ok(stream)
        } else {
            // TODO how to return empty Stream?
            bail!(SdkErrorKind::NoData);
        }
    }
}

pub const TRANSACTION_FIELDS_ORDINARY: &str = r#"
    id
    aborted
    compute {
        compute_type
        skipped_reason
        exit_code
        success
    }
    storage {
       status_change 
    }
    in_msg
    now
    out_msgs
    status
"#;
