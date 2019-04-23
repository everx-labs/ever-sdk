use super::*;
use reql::{Config, Client, Run};
use serde_json::Value;
use reql_types::WriteStatus;

#[test]
#[ignore] // Rethink have to work on 127.0.0.1:32769. Run it and comment "ignore"
fn test_subscribe_updates() {

    // create database, table and record
    let r = Client::new();
    let mut conf = Config::default();
    conf.servers = vec!("127.0.0.1:32769".parse().unwrap());
    let conn = r.connect(conf).unwrap();

    let db_create_res = r.db_create(DB_NAME).run::<Value>(conn).unwrap().wait().next();
    println!("\n\n db_create \n {:#?}", db_create_res);

    let table_create_res = r.db(DB_NAME).table_create(MSG_TABLE_NAME).run::<Value>(conn).unwrap().wait().next();
    println!("\n\n table_create \n {:#?}", table_create_res);

    let msg_id = MessageId::default();

    let insert_doc = r.db(DB_NAME)
        .table(MSG_TABLE_NAME)
        .update( // TODO insert with "update" flag
            json!({ 
                "id": id_to_string(&msg_id),
                MSG_STATE_FIELD_NAME: MessageState::Queued
                })
        )
        .run::<WriteStatus>(conn).unwrap().wait().next().unwrap();
    println!("\n\n insert \n {:#?}", insert_doc);

    // subscribe changes
    
    let changes_stream = Contract::subscribe_updates(conn, msg_id.clone()).unwrap();

    // another thread - write changes into DB
    let msg_id_ = msg_id.clone();
    let another_thread = std::thread::spawn(move || {

        std::thread::sleep(std::time::Duration::from_secs(1));

        for state in [MessageState::Processing, MessageState::Proposed, MessageState::Finalized].iter() {

            let insert_doc = r.db(DB_NAME)
                .table(MSG_TABLE_NAME)
                .replace(json!({
                    "id": id_to_string(&msg_id_),
                    MSG_STATE_FIELD_NAME: state
                 }))
                .run::<WriteStatus>(conn).unwrap().wait().next().unwrap();
            println!("\n\n insert \n {:#?}", insert_doc);
        }
    });

    // chech all changes were got    
    let mut changes_stream = changes_stream.wait();
    for state in [MessageState::Processing, MessageState::Proposed, MessageState::Finalized].iter() {
        let ccs = ContractCallState {
            message_id: msg_id.clone(),
            message_state: state.clone(),
            transaction: None
        };
        
        assert_eq!(changes_stream.next().unwrap().unwrap(), ccs);
    }

    another_thread.join().unwrap();
}
