use crate::*;
use reql::{Client, Connection, Run, Document};
use reql_types::Change;
use futures::stream::{Stream,};
use std::fmt::Debug;
use serde::de::DeserializeOwned;
use reql::{Config};
use std::net::SocketAddr;
use std::sync::Mutex;

lazy_static! {
    static ref RETHINK_CONN: Mutex<Option<Connection>> = Mutex::new(None);
    static ref DB_NAME: Mutex<String> = Mutex::new("blockchain".to_string());
}

// Init global connection to database
pub fn init(config: RethinkConfig) -> SdkResult<()> {
    let mut conn_opt = RETHINK_CONN.lock().unwrap();

    let r = Client::new();
    let mut re_conf = Config::default();
    for s in config.servers.iter() {
        re_conf.servers.push(s.parse::<SocketAddr>()
            .map_err(|_| -> SdkError {
                SdkErrorKind::InvalidArg("error parsing db address".into()).into()
            })?
        );
    }
    *conn_opt = Some(r.connect(re_conf)?);
    if !config.db_name.is_empty() {
        let mut db_name = DB_NAME.lock().unwrap();
        *db_name = config.db_name
    };

    Ok(())
}

pub fn connection() -> SdkResult<Connection> {
    let conn_opt = RETHINK_CONN.lock().unwrap();
    if let Some(conn) = conn_opt.as_ref() {
        Ok(conn.clone())
    } else {
        bail!(SdkErrorKind::NotInitialized)
    }
}

pub fn db() -> Client {
    let db_name = DB_NAME.lock().unwrap();
    Client::new().db(db_name.as_str())
}

// Returns Stream with updates of some field in database
pub fn subscribe_field_updates<T>(table: &str, record_id: &str, field: &str)
    -> SdkResult<Box<dyn Stream<Item = Option<Document<Change<T, T>>>, Error = SdkError>>>
    where T: 'static + Send + DeserializeOwned + Debug {
    let map = db()
        .table(table)
        .get_all(record_id)
        .get_field(field)
        .changes()
        .run::<reql_types::Change<T, T>>(connection()?)?
        .map_err(|err| SdkError::from(err));

    Ok(Box::new(map))
}

// Returns Stream with required database record
pub fn load_record(table: &str, record_id: &str)
    -> SdkResult<Box<Stream<Item=serde_json::Value, Error=SdkError>>> {
    let map = db()
        .table(table)
        .get(record_id)
        .run::<serde_json::Value>(connection()?)?
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
