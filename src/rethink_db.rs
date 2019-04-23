use crate::*;
use reql::{Client, Connection, Run};
use futures::stream::{Stream,};

const DB_NAME: &str = "blockchain";

/*fn subscribe_field_updates<T>(table: &str, record_id: &str, field: &str, db_connection: Connection)     
    -> SdkResult<Box<dyn Stream<Item = Option<Document<Change<T, T>>>, Error = SdkError>>> 
    where T: Send + DeserializeOwned + Debug {

    let r = Client::new();

    let map = r.db(DB_NAME)
        .table(table)
        .get_all(record_id)
        .get_field(field)
        .changes()
        .run::<reql_types::Change<T, T>>(db_connection)?
        .map_err(|err| SdkError::from(err));

    Ok(Box::new(map))
}*/

pub fn load_record(table: &str, record_id: &str, db_connection: Connection)
    -> SdkResult<Box<Stream<Item = serde_json::Value, Error = SdkError>>> {
    
    let r = Client::new();

    let map = r.db(DB_NAME)
        .table(table)
        .get(record_id)
        .run::<serde_json::Value>(db_connection)?
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