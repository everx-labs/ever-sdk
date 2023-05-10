use crate::{error, MessageMonitorSdkServices};
use serde_json::Value;
use ton_types::Cell;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ApiType)]
#[serde(tag = "type", content = "value")]
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

pub(crate) trait CellFromBoc {
    fn convert(&self, boc: &str, name: &str) -> error::Result<Cell>;
}

impl<T: MessageMonitorSdkServices> CellFromBoc for T {
    fn convert(&self, boc: &str, name: &str) -> crate::Result<Cell> {
        self.cell_from_boc(boc, name)
    }
}

impl MonitoredMessage {
    pub(crate) fn hash<Converter: CellFromBoc>(
        &self,
        converter: &Converter,
    ) -> crate::Result<String> {
        Ok(match self {
            MonitoredMessage::HashAddress { hash, .. } => hash.clone(),
            MonitoredMessage::Boc { boc } => converter
                .convert(boc, "message")?
                .repr_hash()
                .as_hex_string(),
        })
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
    /// Helps to identify this message when user received `MessageMonitoringResult`.
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
    /// Returned when the messages was processed and included into finalized block
    /// before `wait_until` block time.
    Finalized,
    /// Returned when the message was not processed until `wait_until` block time.
    Timeout,
    /// Reserved for future statuses. Is never returned.
    /// Application should wait for one of the `Finalized` or `Timeout` statuses.
    /// All other statuses are intermediate.
    Reserved,
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
