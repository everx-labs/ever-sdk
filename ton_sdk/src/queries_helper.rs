use crate::*;
use graphite::client::GqlClient;
use graphite::types::VariableRequest;
use futures::stream::Stream;
use serde_json::Value;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub enum SortDirection {
    #[serde(rename = "ASC")]
    Ascending,
    #[serde(rename = "DESC")]
    Descending
}

#[derive(Serialize, Deserialize)]
pub struct OrderBy {
    path: String,
    direction: SortDirection
}

lazy_static! {
    static ref CLIENT: Mutex<Option<GqlClient>> = Mutex::new(None);
}

// Globally initializes client with server address
pub fn init(config: QueriesConfig) {
    let mut client = CLIENT.lock().unwrap();
    *client = Some(GqlClient::new(&config.queries_server,&config.subscriptions_server));
}

pub fn uninit() {
    let mut client = CLIENT.lock().unwrap();
    *client = None;
}

// Returns Stream with updates of some field in database. First stream item is current value
pub fn subscribe_record_updates(table: &str, filter: &str, fields: &str)
    -> SdkResult<Box<dyn Stream<Item=Value, Error=SdkError>>> {

    let subscription_stream = subscribe(
        table,
        filter,
        fields)?;

    let load_stream = query(table, filter, fields, None, None)?
        .filter(|value| !value[0].is_null())
        .map(|value| value[0].clone());

    Ok(Box::new(load_stream.chain(subscription_stream)))
}

// Returns Stream with updates database fileds by provided filter
pub fn subscribe(table: &str, filter: &str, fields: &str)
    -> SdkResult<Box<dyn Stream<Item=Value, Error=SdkError> + Send>> {

    let request = generate_subscription(table, filter, fields)?;

    let closure_table = table.to_owned();

    let stream = if let Some(client) = CLIENT.lock().unwrap().as_mut() {
         client.subscribe(request)?
            .then(move |result| {
                match result {
                    Err(err) => Err(SdkError::from(err)),
                    Ok(value) => {
                        // try to extract the record value from the answer
                        let record_value = &value["payload"]["data"][&closure_table];
                        
                        if record_value.is_null() {
                            Err(SdkError::from(SdkErrorKind::InvalidData(
                                format!("Invalid subscription answer: {}", value))))
                        } else {
                            Ok(record_value.clone())
                        }
                    }
                }
            })
    } else {
        bail!(SdkErrorKind::NotInitialized)
    };

    Ok(Box::new(stream))
}

// Returns Stream with required database record fields
pub fn load_record_fields(table: &str, record_id: &str, fields: &str)
    -> SdkResult<Box<dyn Stream<Item=Value, Error=SdkError>>> {
    let stream = query(
        table,
        &format!("{{ \"id\": {{\"eq\": \"{record_id}\" }} }}", record_id=record_id),
        fields,
        None,
        None)?
        .and_then(|value| {
            Ok(value[0].clone())
        });

    Ok(Box::new(stream))
}

// Returns Stream with GraphQL query answer 
pub fn query(table: &str, filter: &str, fields: &str, order_by: Option<OrderBy>, limit: Option<u32>)
    -> SdkResult<Box<dyn Stream<Item=Value, Error=SdkError>>> {
    let query = generate_query_var(
        table,
        filter,
        fields,
        order_by,
        limit)?;

    let mut client = CLIENT.lock().unwrap();
    let client = client.as_mut().ok_or(SdkError::from(SdkErrorKind::NotInitialized))?;

    let table = table.to_owned();

    let stream = client.query_vars(query)?
        .then(move |result| {
            match result {
                Err(err) => Err(SdkError::from(err)),
                Ok(value) => {
                    // try to extract the record value from the answer
                    let records_array = &value["data"][&table];
                    if records_array.is_null() {
                        bail!(SdkErrorKind::InvalidData(format!("Invalid query answer: {}", value)))
                    }
                    
                    Ok(records_array.clone())
                }
            }
        });

    Ok(Box::new(stream))
}

// Executes GraphQL query, waits for result and returns recieved value
pub fn wait_for(table: &str, filter: &str, fields: &str) 
    -> SdkResult<Value> {
    let subscription_stream = subscribe(
        table,
        filter,
        fields)?;

    let load_stream = query(table, filter, fields, None, None)?
        .filter(|value| !value[0].is_null())
        .and_then(|value| {
            Ok(value[0].clone())
        });

    Ok(load_stream
        .chain(subscription_stream)
        .wait()
        .next()
        .ok_or(SdkErrorKind::InvalidData("None value".to_owned()))??)
}

fn generate_query_var(table: &str, filter: &str, fields: &str, order_by: Option<OrderBy>, limit: Option<u32>)
    -> SdkResult<VariableRequest>
{
    let mut scheme_type = (&table[0 .. table.len() - 1]).to_owned() + "Filter";
    scheme_type[..1].make_ascii_uppercase();

    let mut query = format!(
        "query {table}($filter: {scheme_type}, $orderBy: [QueryOrderBy], $limit: Int) {{ {table}(filter: $filter, orderBy: $orderBy, limit: $limit) {{ {fields} }}}}",
        table=table,
        scheme_type=scheme_type,
        fields=fields
    );
    query = query.split_whitespace().collect::<Vec<&str>>().join(" ");

    let variables = json!({
        "filter" : serde_json::from_str::<Value>(filter)?,
        "orderBy": order_by,
        "limit": limit
    });

    let variables = variables.to_string().split_whitespace().collect::<Vec<&str>>().join(" ");

    Ok(VariableRequest::new(query, Some(variables)))
}

fn generate_subscription(table: &str, filter: &str, fields: &str) -> SdkResult<VariableRequest> {
    let mut scheme_type = (&table[0 .. table.len() - 1]).to_owned() + "Filter";
    scheme_type[..1].make_ascii_uppercase();

    let query = format!("subscription {table}($filter: {type}) {{ {table}(filter: $filter) {{ {fields} }} }}",
        type=scheme_type,
        table=table,
        fields=fields);
    let query = query.split_whitespace().collect::<Vec<&str>>().join(" ");

    let variables = json!({
        "filter" : serde_json::from_str::<Value>(filter)?
    });
    let variables = variables.to_string().split_whitespace().collect::<Vec<&str>>().join(" ");

    Ok(VariableRequest::new(query, Some(variables)))
}

