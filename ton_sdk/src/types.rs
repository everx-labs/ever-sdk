
// Represents config to connect Rethink DB
#[derive(Debug, Deserialize, Serialize)]
pub struct RethinkConfig {
    pub servers: Vec<String>,
    pub db_name: String,
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
    pub db_config: RethinkConfig,
    pub kafka_config: KafkaConfig,
}