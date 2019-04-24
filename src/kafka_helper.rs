use crate::*;
use std::sync::Mutex;
use kafka::producer::{Producer, Record, RequiredAcks};
use std::time::Duration;

//const COGNFIG_FILE_NAME: &str = "config.json";

lazy_static! {
    static ref CONFIG: KafkaConfig = {
        //let config_json = std::fs::read_to_string(COGNFIG_FILE_NAME).expect("Error reading config");
        //serde_json::from_str(&config_json).expect("Problem parsing config file")

        KafkaConfig{
            servers: vec!("127.0.0.1:9093".into()), // 172.18.0.13
            topic: "requests".into(),
            ack_timeout: 3000,
        }
    };

    static ref KAFKA_PROD: Mutex<Producer> = {
        Mutex::new(
            Producer::from_hosts(CONFIG.servers.clone())
                .with_ack_timeout(Duration::from_millis(CONFIG.ack_timeout))
                .with_required_acks(RequiredAcks::One)
                .create()
                .expect("Problem connecting Kafka")
        )
    };
}

pub fn send_message(key: &[u8], value: &[u8]) -> SdkResult<()> {
    let mut prod = KAFKA_PROD.lock().unwrap();
    prod.send(&Record::from_key_value(&CONFIG.topic, key, value))
        .map_err(|err| err.into())
}