use crate::*;
use graphite::client::GqlClient;
use futures::stream::{Stream,};
use std::fmt::Debug;
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Mutex;

lazy_static! {
    static ref HOST: Mutex<Option<String>> = Mutex::new(None);
    static ref WS_HOST: Mutex<Option<String>> = Mutex::new(None);
}

// Init global connection to database
pub fn init(config: GraphqlConfig) -> SdkResult<()> {
    let mut host = HOST.lock().unwrap();
    let mut ws_host = WS_HOST.lock().unwrap();
    *host = Some(config.host.clone());
    *ws_host = Some(config.socket_host.clone());
    Ok(())
}

pub fn client() -> SdkResult<GqlClient> {
    let host_opt = HOST.lock().unwrap();
    let ws_host_opt = WS_HOST.lock().unwrap();
    
    if host_opt.is_some() && ws_host_opt.is_some() {
        let host = host_opt.clone().unwrap();
        let ws_host = ws_host_opt.clone().unwrap();
        Ok(GqlClient::new(host.to_string(), ws_host.to_string()))
    } else {
        bail!(SdkErrorKind::NotInitialized)
    }
}

// Returns Stream with updates of some field in database
pub fn subscribe_field_updates(table: &str, record_id: &str)
    -> SdkResult<Box<dyn Stream<Item=Value, Error=SdkError>>> {
    
    let mut client = client()?;
    let query = generate_subscription(table, record_id)?;
    let stream = client.subscribe(query).map_err(|err| SdkError::from(err));
    
    Ok(Box::new(stream))
}

// Returns Stream with required database record
pub fn load_record(table: &str, record_id: &str)
    -> SdkResult<Box<dyn Stream<Item=Value, Error=SdkError>>> {
    
    let client = client()?;
    let query = generate_query(table, record_id)?;
    let stream = client.query(query).map_err(|err| SdkError::from(err));
        
    return Ok(Box::new(stream));    
}

fn generate_query(table: &str, record_id: &str) -> SdkResult<String> {
    create_query_template(table)
        .map(|(structure, fields)| {
            format!("{{{structure}{{{table}(id: \"{record_id}\"){{{fields}}}}}}}", 
                structure=structure, 
                table=table, 
                record_id=record_id, 
                fields=fields)
        })
}

fn generate_subscription(table: &str, record_id: &str) -> SdkResult<String> {
    create_query_template(table)
        .map(|(structure, fields)| {
            format!("subscription {structure}{{{table}(id: \\\"{record_id}\\\"){{{fields}}}}}", 
                structure=structure, 
                table=table, 
                record_id=record_id, 
                fields=fields)
        })
}

fn create_query_template(table: &str) -> SdkResult<(String, String)> {
    return match table {
        BLOCKS_TABLE_NAME => Ok(("Block".to_string(), "id, status".to_string())),
        CONTRACTS_TABLE_NAME => Ok(("Account".to_string(), "id, storage".to_string())),
        MESSAGES_TABLE_NAME => Ok(("Message".to_string(), "id, status, body, block".to_string())),
        TRANSACTIONS_TABLE_NAME => Ok(("Transaction".to_string(), "id, status, in_msg, out_msgs, aborted, block, account".to_string())),
        _ => bail!(SdkErrorKind::InvalidArg("Unknown table name".to_string()))
    }
}
