use crate::*;
use std::sync::Mutex;
/*use kafka::producer::{Producer, Record, RequiredAcks};
use std::time::Duration;


lazy_static! {
    static ref KAFKA_PROD: Mutex<Option<(Producer, KafkaConfig)>> = Mutex::new(None);
}

// Init global variable - kafka config
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

// Puts message into Kafka (topic name is globally configured by init func)
pub fn send_message(key: &[u8], value: &[u8]) -> SdkResult<()> {
    let mut prod_opt = KAFKA_PROD.lock().unwrap();
    if let Some((prod, config)) = prod_opt.as_mut() {
        prod.send(&Record::from_key_value(&config.topic, key, value))
            .map_err(|err| err.into())
    } else {
        bail!(SdkErrorKind::NotInitialized);
    }
}

// Puts message into Kafka topic eith given name
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
*/
/*
// Init global variable - kafka config
lazy_static! {
    static ref KAFKA_PROD: Mutex<Option<KafkaConfig>> = Mutex::new(None);
}

pub fn init(config: KafkaConfig) -> SdkResult<()> {
    let mut prod_opt = KAFKA_PROD.lock().unwrap();
    *prod_opt = config;
    Ok(())
}
*/

//Using kafka via HTTP REST PROXY!!!

extern crate reqwest;
extern crate base64;

use self::reqwest::Client;
use self::reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::json;
use std::io::Read;

lazy_static! {
    static ref CONFIG: Mutex<Option<KafkaConfig>> = Mutex::new(None);
    static ref CLIENT: Mutex<Option<Client>> = Mutex::new(None);
}

pub fn init(kafka_config: KafkaConfig) -> SdkResult<()> {
    let mut config = CONFIG.lock().unwrap();
    let mut client = CLIENT.lock().unwrap();
    *config = Some(kafka_config);
    *client = Some(Client::new());

    Ok(())
}

// Puts message into Kafka (topic name is globally configured by init func)
pub fn send_message(key: &[u8], value: &[u8]) -> SdkResult<()> {
    let client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let key_encoded = base64::encode(key);
    let value_encoded = base64::encode(value);
    let body = json!({
        "records": [{ "key": key_encoded, "value": value_encoded }]
    });

    let result = client.post("https://services.tonlabs.io/topics/requests")
        .headers(headers)
        .body(body.to_string())
        .send();
    match result {
        Ok(result) => {
            if result.status().is_success() {
                Ok(())
            } else {
                let bytes: Vec<u8> = result.bytes().map(|b| if let Ok(b) = b { b } else { 0 }).collect();
                let text = match String::from_utf8(bytes.clone()) {
                    Ok(text) => text,
                    Err(_) => hex::encode(bytes)
                };
                bail!(SdkErrorKind::InternalError(format!("Request failed: {}", text)))
            }
        }
        Err(err) => bail!(SdkErrorKind::InternalError(format!("Can not send request: {}", err)))
    }
}

// Puts message into Kafka topic eith given name
#[allow(dead_code)]
pub fn send_message_to_topic(key: &[u8], value: &[u8], topic: &str) -> SdkResult<()> {
    let client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let key_encoded = base64::encode(key);
    let value_encoded = base64::encode(value);
    let body = json!({
        "records": [{ "key": key_encoded, "value": value_encoded }]
    });

    let url = format!("https://services.tonlabs.io/topics/{}", &topic);
    let result = client.post(&url)
        .headers(headers)
        .body(body.to_string())
        .send();

    if result.is_err() {
        bail!(SdkErrorKind::InternalError("Kafka send error".to_string()));
    } else {
        Ok(())
    }
}

