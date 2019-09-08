pub const MESSAGES_TABLE_NAME: &str = "messages";
pub const MESSAGES_FILTER_NAME: &str = "MessageFilter";
pub const CONTRACTS_TABLE_NAME: &str = "accounts";
pub const CONTRACTS_FILTER_NAME: &str = "AccountFilter";
pub const BLOCKS_TABLE_NAME: &str = "blocks";
pub const BLOCKS_FILTER_NAME: &str = "BlockFilter";
pub const TRANSACTIONS_TABLE_NAME: &str = "transactions";
pub const TRANSACTIONS_FILTER_NAME: &str = "TransactionFilter";


pub const CONTRACT_CALL_STATE_FIELDS: &str = "id status";

pub const MSG_STATE_FIELD_NAME: &str = "status";

// Represents config to connect with Rethink DB and Kafka
#[derive(Debug, Deserialize, Serialize)]
pub struct QueriesConfig {
    pub queries_server: String,
    pub subscriptions_server: String,
}

// Represents config to connect with Rethink DB and Kafka
#[derive(Debug, Deserialize, Serialize)]
pub struct RequestsConfig {
    pub requests_server: String,
}

// Represents config to connect with Rethink DB and Kafka
#[derive(Debug, Deserialize, Serialize)]
pub struct NodeClientConfig {
    pub queries_config: QueriesConfig,
    pub requests_config: RequestsConfig,
}
