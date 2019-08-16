pub const MESSAGES_TABLE_NAME: &str = "messages";
pub const CONTRACTS_TABLE_NAME: &str = "accounts";
pub const BLOCKS_TABLE_NAME: &str = "blocks";
pub const TRANSACTIONS_TABLE_NAME: &str = "transactions";

pub const CONTRACT_CALL_STATE_FIELDS: &str = "id status";

pub const MSG_STATE_FIELD_NAME: &str = "status";

// Represents config to connect with Rethink DB and Kafka
#[derive(Debug, Deserialize, Serialize)]
pub struct NodeClientConfig {
    pub queries_server: String,
    pub requests_server: String,
}