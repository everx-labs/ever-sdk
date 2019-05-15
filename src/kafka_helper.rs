use crate::*;
use std::sync::Mutex;
use kafka::producer::{Producer, Record, RequiredAcks};
use std::time::Duration;

//     KafkaConfig{
//         servers: vec!("127.0.0.1:9092".into()),
//         topic: "requests".into(),
//         ack_timeout: 3000,
//     }

lazy_static! {
    static ref KAFKA_PROD: Mutex<Option<(Producer, KafkaConfig)>> = Mutex::new(None);
}

pub fn init(config: KafkaConfig) -> SdkResult<()> {
    let mut prod_opt = KAFKA_PROD.lock().unwrap();
    *prod_opt = Some((
            Producer::from_hosts(config.servers.clone())
                .with_ack_timeout(Duration::from_millis(config.ack_timeout))
                .with_required_acks(RequiredAcks::One)
                .create()?,
            config
        ));
    Ok(())
}

pub fn send_message(key: &[u8], value: &[u8]) -> SdkResult<()> {
    let mut prod_opt = KAFKA_PROD.lock().unwrap();
    if let Some((prod, config)) = prod_opt.as_mut() { 
        prod.send(&Record::from_key_value(&config.topic, key, value))
            .map_err(|err| err.into())
    } else {
        bail!(SdkErrorKind::NotInitialized);
    }
}

#[allow(dead_code)]
pub fn send_message_to_topic(key: &[u8], value: &[u8], topic: &str) -> SdkResult<()> {
    let mut prod_opt = KAFKA_PROD.lock().unwrap();
    if let Some((prod, _)) = prod_opt.as_mut() { 
        prod.send(&Record::from_key_value(topic, key, value))
            .map_err(|err| err.into())
    } else {
        bail!(SdkErrorKind::NotInitialized);
    }
}
