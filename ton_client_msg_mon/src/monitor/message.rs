use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ApiType)]
pub struct MessageMonitoringParams {
    /// Hash of the message that was sent to the blockchain.
    pub hash: String,

    /// Destination account address
    pub address: String,

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
