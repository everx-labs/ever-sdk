use crate::*;
use graphite::client::GqlClient;
use graphite::types::VariableRequest;
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

fn rename_key_to_id(value: serde_json::Value) -> SdkResult<serde_json::Value> {
    if let serde_json::Value::Object(mut obj) = value {
        let id = obj.get("_key").map(|v| v.clone());
        if let Some(id) = id {
            obj.insert("id".to_string(), id);
            obj.remove("_key");
            Ok(serde_json::Value::Object(obj))
        } else {
            bail!(SdkErrorKind::InvalidData("rename_key_to_id: id not found".into()))
        }
    } else {
        bail!(SdkErrorKind::InvalidData("rename_key_to_id: invalid json value".into()))
    }
}

// Returns Stream with required database record
pub fn load_record(table: &str, record_id: &str)
    -> SdkResult<Box<dyn Stream<Item=Value, Error=SdkError>>> {

    let mut client = client()?;
    let query = generate_select(table, record_id)?;
    let stream = client.subscribe(query)
        .map(|value| {
            let tr_value: serde_json::Value = serde_json::from_str(value["payload"]["data"]["select"].as_str().unwrap())
                .unwrap();

            println!("value {}", tr_value[0].to_string());
            rename_key_to_id(tr_value[0].clone()).unwrap()
        })
        .map_err(|err| SdkError::from(err));

    /*let query = generate_query(table, record_id)?;
    println!("query {}", query);
    let stream = client.query(query).map_err(|err| SdkError::from(err));*/

    return Ok(Box::new(stream));
}

fn generate_query(table: &str, record_id: &str) -> SdkResult<String> {
    create_query_template(table)
        .map(|(structure, fields)| {
            format!("query {structure} {{ {structure}(filter: \"{{ \\\"match\\\": {{ \\\"id\\\": \\\"{record_id}\\\" }} }}\") {{ {fields} }} }}",
                structure=structure,
                record_id=record_id,
                fields=fields)
        })
}

fn generate_select(table: &str, record_id: &str) -> SdkResult<VariableRequest> {
    create_query_template(table)
        .map(|(structure, _)| {
            let query = "query select($query: String!, $bindVarsJson: String!) {select(query: $query, bindVarsJson: $bindVarsJson)}".to_owned();

            let db_query = format!("RETURN DOCUMENT(\"{}/{}\")", structure, record_id);

            let variables = json!({"query" : db_query,"bindVarsJson": "{}"});

            println!("variables {}", variables.to_string());

            return VariableRequest::new(query, Some(variables.to_string()));
        })
}

fn generate_subscription(table: &str, record_id: &str) -> SdkResult<VariableRequest> {
    create_query_template(table)
        .map(|(structure, fields)| {
            let query = format!("subscription {structure}($match: String) {{ {structure}(match: $match) {{ {fields} }} }}",
                structure=structure,
                fields=fields);

            println!("query {}", query);

            let variables = format!("{{\"match\":\"{{\\\"id\\\":\\\"{record_id}\\\"}}\"}}",
                record_id=record_id);

            println!("variables {}", variables);

            return VariableRequest::new(query, Some(variables));
        })
}

fn create_query_template(table: &str) -> SdkResult<(String, String)> {
    return match table {
        BLOCKS_TABLE_NAME => Ok(("blocks".to_string(), "id, status".to_string())),
        CONTRACTS_TABLE_NAME => Ok(("accounts".to_string(), "id, storage".to_string())),
        MESSAGES_TABLE_NAME => Ok(("messages".to_string(), "id, status, body".to_string())),
        TRANSACTIONS_TABLE_NAME => Ok(("transactions".to_string(), "id, status, in_msg, out_msgs, aborted, account_addr".to_string())),
        _ => bail!(SdkErrorKind::InvalidArg("Unknown table name".to_string()))
    }
}
