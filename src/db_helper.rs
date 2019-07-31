use crate::*;
use graphite::client::GqlClient;
use graphite::types::SubscribeRequest;
use futures::stream::Stream;
use serde_json::Value;
use std::sync::Mutex;

lazy_static! {
    static ref HOST: Mutex<Option<String>> = Mutex::new(None);
}

// Init global connection to database
pub fn init(config: GraphqlConfig) {
    let mut host = HOST.lock().unwrap();
    *host = Some(config.server.clone());
}

pub fn client() -> SdkResult<GqlClient> {
    let host_opt = HOST.lock().unwrap();
    
    if host_opt.is_some() {
        let host = host_opt.clone().unwrap();
        Ok(GqlClient::new(host.to_string()))
    } else {
        bail!(SdkErrorKind::NotInitialized)
    }
}

// Returns Stream with updates of some field in database
pub fn subscribe_field_updates(table: &str, record_id: &str)
    -> SdkResult<Box<dyn Stream<Item=Value, Error=SdkError>>> {
    
    let mut client = client()?;
    let request = generate_subscription(table, record_id)?;
    let stream = client.subscribe(request).map_err(|err| SdkError::from(err));
    
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
            format!("query {structure} {{ {structure}(filter: \"{{ \\\"id\\\": \\\"{record_id}\\\" }}\") {{ {fields} }} }}", 
                structure=structure, 
                record_id=record_id,
                fields=fields)
        })
}

fn generate_subscription(table: &str, record_id: &str) -> SdkResult<SubscribeRequest> {
    create_query_template(table)
        .map(|(structure, fields)| {
            let query = format!("subscription {structure}($match: String) {{ {structure}(match: $match) {{ {fields} }} }}", 
                structure=structure, 
                fields=fields);
                
            let variables = format!("{{\"match\":\"{{\\\"id\\\":\\\"{record_id}\\\"}}\"}}", 
                record_id=record_id);
            
            return SubscribeRequest::new(query, Some(variables));    
        })
}

fn create_query_template(table: &str) -> SdkResult<(String, String)> {
    return match table {
        BLOCKS_TABLE_NAME => Ok(("blocks".to_string(), "id, status".to_string())),
        CONTRACTS_TABLE_NAME => Ok(("accounts".to_string(), "id, storage".to_string())),
        MESSAGES_TABLE_NAME => Ok(("messages".to_string(), "id, status, body, block".to_string())),
        TRANSACTIONS_TABLE_NAME => Ok(("transactions".to_string(), "id, status, in_msg, out_msgs, aborted, block, account".to_string())),
        _ => bail!(SdkErrorKind::InvalidArg("Unknown table name".to_string()))
    }
}
