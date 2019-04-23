
#[derive(Debug, Deserialize, Serialize)]
pub struct RethinkConfig {
    pub servers: Vec<String>,
    pub db_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KafkaConfig {
    pub servers: Vec<String>,
    pub topic: String,
    pub ack_timeout: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NodeClientConfig {
    pub db_config: RethinkConfig,
    pub kafka_config: KafkaConfig,
}