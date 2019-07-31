pub const MESSAGES_TABLE_NAME: &str = "messages";
pub const CONTRACTS_TABLE_NAME: &str = "accounts";
pub const BLOCKS_TABLE_NAME: &str = "blocks";
pub const TRANSACTIONS_TABLE_NAME: &str = "transactions";

// Represents config to connect Rethink DB
#[derive(Debug, Deserialize, Serialize)]
pub struct GraphqlConfig {
    pub server: String,
}

// Represents config to connect Kafka
#[derive(Debug, Deserialize, Serialize)]
pub struct KafkaConfig {
    pub servers: Vec<String>,
    pub topic: String,
    pub ack_timeout: u64,
}

// Represents config to connect with Rethink DB and Kafka
#[derive(Debug, Deserialize, Serialize)]
pub struct NodeClientConfig {
    pub graphql_config: GraphqlConfig,
    pub kafka_config: KafkaConfig,
}