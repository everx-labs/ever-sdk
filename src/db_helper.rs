use crate::*;
use reql::{Client, Connection, Run, Document};
use reql_types::Change;
use futures::stream::{Stream,};
use std::fmt::Debug;
use serde::de::DeserializeOwned;
use reql::{Config};
use std::net::SocketAddr;

//const COGNFIG_FILE_NAME: &str = "config.json";
const DB_NAME: &str = "blockchain";

lazy_static! {
    static ref CONFIG: KafkaConfig = {
        //let config_json = std::fs::read_to_string(COGNFIG_FILE_NAME).expect("Error reading config");
        //serde_json::from_str(&config_json).expect("Problem parsing config file")

        KafkaConfig{
            servers: vec!("127.0.0.1:32769".into()),
            topic: "messages".into(),
            ack_timeout: 3000,
        }
    };
    
    static ref RETHINK_CONN: Connection = {
        let r = Client::new();
        let mut conf = Config::default();
        for s in CONFIG.servers.iter() {
            conf.servers.push(s.parse::<SocketAddr>().expect("Error parsing address"));
        }
         r.connect(conf).unwrap()
    };
}

pub fn subscribe_field_updates<T>(table: &str, record_id: &str, field: &str)
    -> SdkResult<Box<dyn Stream<Item = Option<Document<Change<T, T>>>, Error = SdkError>>> 
    where T: 'static + Send + DeserializeOwned + Debug {

    let r = Client::new();

    let map = r.db(DB_NAME)
        .table(table)
        .get_all(record_id)
        .get_field(field)
        .changes()
        .run::<reql_types::Change<T, T>>(RETHINK_CONN.clone())?
        .map_err(|err| SdkError::from(err));

    Ok(Box::new(map))
}

pub fn load_record(table: &str, record_id: &str)
    -> SdkResult<Box<Stream<Item = serde_json::Value, Error = SdkError>>> {
    
    let r = Client::new();

    let map = r.db(DB_NAME)
        .table(table)
        .get(record_id)
        .run::<serde_json::Value>(RETHINK_CONN.clone())?
        .map(move |arr_opt| {
            if let Some(reql::Document::Expected(serde_json::Value::Array(arr))) = arr_opt {
                if let Some(val) = arr.get(0) {
                    return val.clone();
                }
            }
            return json!(null);
        })
        .map_err(|err| SdkError::from(err));

    Ok(Box::new(map))
}