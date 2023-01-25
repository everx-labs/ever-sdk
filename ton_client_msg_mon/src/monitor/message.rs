use serde_json::Value;
use ton_block::MsgAddrStd;
use ton_types::UInt256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageMonitoringParams {
    pub hash: UInt256,
    pub address: MsgAddrStd,
    pub wait_until: u32,
    pub user_data: Option<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageMonitoringResult {
    pub hash: UInt256,
    pub user_data: Option<Value>,
    pub status: MessageMonitoringStatus,
    pub transaction: Option<MessageMonitoringTransaction>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageMonitoringStatus {
    Finalized,
    Timeout,
    Reserved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageMonitoringTransaction {
    pub hash: UInt256,
    pub aborted: bool,
    pub compute: Option<MessageMonitoringTransactionCompute>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageMonitoringTransactionCompute {
    pub exit_code: i32,
}
