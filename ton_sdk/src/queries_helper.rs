/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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
use graphite::types::{VariableRequest};
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

const DEFAULT_TIMEOUT: u32 = 40000;

lazy_static! {
    static ref CLIENT: Mutex<Option<GqlClient>> = Mutex::new(None);
}

fn check_redirect(address: &str) -> SdkResult<Option<String>> {
    let client = ClientBuilder::new()
        .redirect(RedirectPolicy::none())
        .build()
        .map_err(|err| SdkErrorKind::InternalError { msg: format!("Can not build test request: {}", err) } )?;

    let result = client.get(address).send();

    match result {
        Ok(result) => {
            if result.status() == StatusCode::PERMANENT_REDIRECT {
                let address = result
                    .headers()
                    .get(LOCATION)
                    .ok_or(SdkErrorKind::NetworkError { msg: "Missing location field in redirect response".to_owned() } )?
                    .to_str()
                    .map_err(|err| SdkErrorKind::NetworkError { msg: format!("Can not cast redirect location to string: {}", err) } )?
                    .to_owned();

                Ok(Some(address))
            } else {
                Ok(None)
            }
        },
        Err(err) => bail!(SdkErrorKind::NetworkError { msg: format!("Can not send test request: {}", err) } )
    }
}

// Globally initializes client with server address
pub fn init(mut config: NodeClientConfig) -> SdkResult<()> {
    if let Some(redirected) = check_redirect(&config.queries_server)? {
        config = NodeClientConfig {
            queries_server: redirected.clone(),
            subscriptions_server: redirected
                .replace("https://", "wss://")
                .replace("http://", "ws://")
        }
    }
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
    -> SdkResult<Box<dyn Stream<Item=Value, Error=SdkError> + Send>> {

    let subscription_stream = subscribe(
        table,
        filter,
        fields)?;

    let load_stream = query(table, filter, fields, None, None, None)?
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
                            Err(SdkError::from(SdkErrorKind::InvalidData {
                                msg: format!("Invalid subscription answer: {}", value)
                            }))
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
    -> SdkResult<Box<dyn Stream<Item=Value, Error=SdkError> + Send>> {
    let stream = query(
        table,
        &format!("{{ \"id\": {{\"eq\": \"{record_id}\" }} }}", record_id=record_id),
        fields,
        None,
        None,
        None)?
        .and_then(|value| {
            Ok(value[0].clone())
        });

    Ok(Box::new(stream))
}

// Returns Stream with GraphQL query answer 
pub fn query(
    table: &str,
    filter: &str,
    fields: &str,
    order_by: Option<OrderBy>,
    limit: Option<u32>,
    timeout: Option<u32>
) -> SdkResult<Box<dyn Stream<Item=Value, Error=SdkError> + Send>> {
    let query = generate_query_var(
        table,
        filter,
        fields,
        order_by,
        limit,
        timeout)?;

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
                        Err(SdkError::from(
                            SdkErrorKind::InvalidData { msg: format!("Invalid query answer: {}", value) }
                        ))
                    } else {
                        Ok(records_array.clone())
                    }
                }
            }
        });

    Ok(Box::new(stream))
}

// Executes GraphQL query, waits for result and returns recieved value
pub fn wait_for(table: &str, filter: &str, fields: &str, timeout: Option<u32>)
    -> SdkResult<Box<dyn Stream<Item=Value, Error=SdkError> + Send>>
{
    let timeout = match timeout {
        Some(timeout) => timeout,
        None => DEFAULT_TIMEOUT
    };

    let stream = query(table, filter, fields, None, None, Some(timeout))?
        .and_then(|value| {
            if !value[0].is_null() {
                Ok(value[0].clone())
            } else {
                Err(SdkErrorKind::WaitForTimeout.into())
            }
            
        });

    Ok(Box::new(stream))
}

fn generate_query_var(
    table: &str,
    filter: &str,
    fields: &str,
    order_by: Option<OrderBy>,
    limit: Option<u32>,
    timeout: Option<u32>
) -> SdkResult<VariableRequest> {
    let mut scheme_type = (&table[0 .. table.len() - 1]).to_owned() + "Filter";
    scheme_type[..1].make_ascii_uppercase();

    let mut query = format!(
        r#"query {table}
        ($filter: {scheme_type}, $orderBy: [QueryOrderBy], $limit: Int, $timeout: Float)
        {{ 
            {table}(filter: $filter, orderBy: $orderBy, limit: $limit, timeout: $timeout)
            {{ {fields} }}
        }}"#,
        table=table,
        scheme_type=scheme_type,
        fields=fields
    );
    query = query.split_whitespace().collect::<Vec<&str>>().join(" ");

    let variables = json!({
        "filter" : serde_json::from_str::<Value>(filter)?,
        "orderBy": order_by,
        "limit": limit,
        "timeout": timeout
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

#[derive(Debug, Clone, Serialize)]
pub struct MutationRequest {
    pub id: String,
    pub body: String
}

fn generate_post_mutation(requests: &[MutationRequest]) -> SdkResult<VariableRequest> {
    let query = "mutation postRequests($requests:[Request]){postRequests(requests:$requests)}".to_owned();
    let variables = json!({
        "requests": serde_json::to_value(requests)?
    }).to_string();

    Ok(VariableRequest::new(query, Some(variables)))
}

// Sends message to node
pub fn send_message(key: &[u8], value: &[u8]) -> SdkResult<()> {
    if let Some(client) = CLIENT.lock().unwrap().as_ref() {
        let request = MutationRequest {
            id: base64::encode(key),
            body: base64::encode(value)
        };

        Ok(client.query_vars(generate_post_mutation(&[request])?)?
            .wait()
            .next()
            .ok_or(SdkErrorKind::NetworkError {
                    msg: "Post message error: server did not responded".to_owned()
                })?
            .map(|_| ())?
        )
    } else {
        bail!(SdkErrorKind::NotInitialized);
    }
}
