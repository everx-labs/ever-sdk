/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::*;
use graphite::client::GqlClient;
use graphite::types::VariableRequest;
use futures::stream::Stream;
use serde_json::Value;
use std::sync::Mutex;
use reqwest::{ClientBuilder, RedirectPolicy, StatusCode};
use reqwest::header::LOCATION;

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

fn check_redirect(config: QueriesConfig) -> SdkResult<QueriesConfig> {
    let client = ClientBuilder::new()
        .redirect(RedirectPolicy::none())
        .build()
        .map_err(|err| SdkErrorKind::InternalError(format!("Can not build test request: {}", err)))?;

    let result = client.get(&config.queries_server).send();

    match result {
        Ok(result) => {
            if result.status() == StatusCode::PERMANENT_REDIRECT {
                let address = result
                    .headers()
                    .get(LOCATION)
                    .ok_or(SdkErrorKind::NetworkError("Missing location field in redirect response".to_owned()))?
                    .to_str()
                    .map_err(|err| SdkErrorKind::NetworkError(format!("Can not cast redirect location to string: {}", err)))?
                    .to_owned();
                
                Ok(QueriesConfig {
                    queries_server: address.clone(),
                    subscriptions_server: address
                        .replace("https://", "wss://")
                        .replace("http://", "ws://")
                })
            } else {
                Ok(config)
            }
        },
        Err(err) => bail!(SdkErrorKind::NetworkError(format!("Can not send test request: {}", err)))
    }
}

// Globally initializes client with server address
pub fn init(config: QueriesConfig) -> SdkResult<()> {
    let config = check_redirect(config)?;
    let mut client = CLIENT.lock().unwrap();
    *client = Some(GqlClient::new(&config.queries_server,&config.subscriptions_server));
    Ok(())
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

