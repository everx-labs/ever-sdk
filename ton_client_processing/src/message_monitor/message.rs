use crate::Error;
use base64::Engine;
use serde_json::Value;
use std::io::Cursor;
use ton_types::{deserialize_tree_of_cells, UInt256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ApiType)]
pub enum MonitoredMessage {
    /// BOC of the message.
    Boc { boc: String },
    /// Message's hash and destination address.
    HashAddress {
        /// Hash of the message.
        hash: String,
        /// Destination address of the message.
        address: String,
    },
}

impl MonitoredMessage {
    pub fn hash(&self) -> crate::Result<String> {
        Ok(match self {
            MonitoredMessage::HashAddress { hash, .. } => hash.clone(),
            MonitoredMessage::Boc { boc } => Self::get_boc_hash(boc)?.as_hex_string(),
        })
    }

    pub fn get_boc_hash(boc: &str) -> crate::Result<UInt256> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(boc)
            .map_err(|err| {
                Error::invalid_boc(format!("error decode message BOC base64: {}", err))
            })?;
        let cell = deserialize_tree_of_cells(&mut Cursor::new(&bytes)).map_err(|err| {
            Error::invalid_boc(format!("Message BOC deserialization error: {}", err))
        })?;

        Ok(cell.repr_hash())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ApiType)]
pub struct MessageMonitoringParams {
    /// Monitored message identification.
    /// Can be provided as a message's BOC or (hash, address) pair.
    /// BOC is a preferable way because it helps to determine possible error reason (using TVM
    /// execution of the message).
    pub message: MonitoredMessage,

    /// Block time
    /// Must be specified as a UNIX timestamp in seconds
    pub wait_until: u32,

    /// User defined data associated with this message.
    /// Helps to identify this message when user received message processing results.
    pub user_data: Option<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ApiType)]
pub struct MessageMonitoringResult {
    /// Hash of the message.
    pub hash: String,

    /// Processing status.
    pub status: MessageMonitoringStatus,

    /// In case of `Finalized` the transaction is extracted from the block.
    /// In case of `Timeout` the transaction is emulated using the last known
    /// account state.
    pub transaction: Option<MessageMonitoringTransaction>,

    /// In case of `Timeout` contains possible error reason.
    pub error: Option<String>,

    /// User defined data related to this message.
    /// This is the same value as passed before with `MessageMonitoringParams` or `SendMessageParams`.
    pub user_data: Option<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ApiType)]
pub enum MessageMonitoringStatus {
    /// The message was processed on the blockchain and transaction was included into
    /// finalized block.
    /// This is terminal status for message monitoring.
    Finalized,
    /// The message (and transaction) was not processed on the blockchain before the
    /// specified `wait_until` time.
    /// In other words – there are no blocks containing transaction and with `gen_utime`
    /// less or equal to the `wait_until`.
    /// This is terminal status for message monitoring.
    Timeout,
    /// Full node tries to execute message onto the actual shard state and encounters error.
    /// So this message was not sent to the validators.
    /// It is an intermediate status. Next status will be a `Timeout`.
    RejectedByFullNode,
    /// Indicates that message was included into shard block but not finalized by masterchain yet.
    /// This status is reported by REMP protocol.
    /// It is an intermediate status. Next status could be a `Finalized` (most possible)
    /// or `Timeout`.
    IncludedIntoBlock,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ApiType)]
pub struct MessageMonitoringTransaction {
    /// Hash of the transaction.
    /// Present if transaction was included into the blocks.
    /// When then transaction was emulated this field will be missing.
    pub hash: Option<String>,
    /// Aborted field of the transaction.
    pub aborted: bool,
    /// Optional information about the compute phase of the transaction.
    pub compute: Option<MessageMonitoringTransactionCompute>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ApiType)]
pub struct MessageMonitoringTransactionCompute {
    /// Compute phase exit code.
    pub exit_code: i32,
}
