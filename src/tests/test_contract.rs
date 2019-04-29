use abi_lib_dynamic::json_abi::decode_function_responce;
use super::*;
use reql::{Config, Client, Run};
use serde_json::Value;
use reql_types::WriteStatus;
use tvm::types::AccountId;
use tvm::stack::{BuilderData, IBitstring};

const DB_NAME: &str = "blockchain";

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
    let changes_stream = Contract::subscribe_updates(msg_id.clone()).unwrap();

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

#[test]
#[ignore] 
fn test_subscribe_updates_kafka_connector() {

    /* Connector config

connector.class=com.datamountaineer.streamreactor.connect.rethink.sink.ReThinkSinkConnector
tasks.max=1
topics=messages_statuses
connect.rethink.db=blockchain
connect.rethink.host=rethinkdb
connect.rethink.port=28015
key.converter.schemas.enable=false
name=rethink-sink
value.converter.schemas.enable=false
value.converter=org.apache.kafka.connect.json.JsonConverter
key.converter=org.apache.kafka.connect.json.JsonConverter
connect.rethink.kcql=UPSERT INTO messages_statuses SELECT * FROM messages_statuses AUTOCREATE PK message_id

    */


    // init SDK
    let config_json = r#"
        {
            "db_config": {
                "servers": ["127.0.0.1:28015"],
                "db_name": "some name"
            },
            "kafka_config": {
                "servers": ["127.0.0.1:9092"],
                "topic": "requests",
                "ack_timeout": 1000
            }
        }"#;    
    init_json(config_json.into()).unwrap();


    let msg_id = MessageId::default();

    // subscribe changes
    let changes_stream = Contract::subscribe_updates(msg_id.clone()).unwrap();

    // another thread - write changes into DB though Kafka (emulate node activity)
    let msg_id_ = msg_id.clone();
    let another_thread = std::thread::spawn(move || {

        std::thread::sleep(std::time::Duration::from_secs(1));

        for state in [MessageState::Processing, MessageState::Proposed, MessageState::Finalized].iter() {
            let key = format!("\"{}\"", id_to_string(&msg_id_));
            
            let doc = json!({
                "message_id": id_to_string(&msg_id_),
                MSG_STATE_FIELD_NAME: state
            }).to_string();
            
            kafka_helper::send_message_to_topic(
                    key.as_bytes(),
                    doc.as_bytes(),
                    "messages_statuses"
                )
                .unwrap();

            println!("NODE {}", doc);
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

        let json = serde_json::to_string(&ccs).unwrap();
        println!("CLIENT {}", json);

        assert_eq!(changes_stream.next().unwrap().unwrap(), ccs);
    }

    another_thread.join().unwrap();
}

#[test]
#[ignore] 
fn test_call_contract() {

    let id = AccountId::from([11; 32]);
    let func = "".to_string(); // TODO
    let input = "".to_string(); // TODO
    let abi = "".to_string(); // TODO
    
    let key_bytes = std::fs::read("key-pair")
        .expect("Problem reading keyfile");
    
    let key_pair = Keypair::from_bytes(&key_bytes)
        .expect("Problem parsing keyfile");

    let contract = Contract::load(id)
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract");

    // call needed method
    let changes_stream = contract.call_json(func.clone(), input, abi.clone(), Some(&key_pair))
        .expect("Error calling contract method");

    // wait transaction id in message-status 
    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            println!("next state: {:?}", s);
            if let Some(id) = s.transaction {
                tr_id = Some(id);
            }
            if s.message_state == MessageState::Finalized {
                break;
            }
        }
    }
    let tr_id = tr_id.expect("Error: no transaction id");

    // OR 
    // wait message will done and find transaction with the message

    // load transaction object
    let tr = Transaction::load(tr_id)
        .expect("Error calling load Transaction")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Transaction")
        .expect("Error unwrap result while loading Transaction");

    // take external outbound message from the transaction
    let out_msg = tr.load_out_messages()
        .expect("Error calling load out messages")
        .wait()
        .find(|msg| msg.as_ref().expect("erro unwrap out message").msg_type() == MessageType::OutboundExternal)
            .expect("erro unwrap out message 2")
            .expect("erro unwrap out message 3");

    // take body from the message
    let responce = out_msg.body().into();

    // decode the body by ABI
    let result = decode_function_responce(abi, func, responce)
        .expect("Error decoding result");

    println!("result:/n{}", result);


    // this way it is need:
    // 1. message status with transaction id or transaction object with in-message id
    // 2. transaction object with out messages ids
    // 3. message object with body

}

#[test]
#[ignore]
fn test_deploy_contract() {
    // TODO

    // read image from file

    // construct image

    // call deploy method

    // wait transaction id in message-status or 
    // wait message will done and find transaction with the message

    // load transaction object

    // take external outbound message from the transaction

    // take body from the message

    // decode the body by ABI
}

#[test]
fn test_send_empty_messages() {
    let id = AccountId::from([11; 32]);
    let contract = Contract { id };
    
    let config_json = r#"
    {
        "servers": ["127.0.0.1:9092"],
        "topic": "kirill-test",
        "ack_timeout": 1000
    }"#;

    let config : KafkaConfig = serde_json::from_str(&config_json).unwrap();

    kafka_helper::init(config).unwrap();

    for i in 0..10 {
        // fake body
        let mut builder = BuilderData::default();
        builder.append_u32(i).unwrap();
        let msg_body = builder.into();
        
        let msg = contract.create_message(msg_body).unwrap();

        // send message by Kafka
        let msg_id = Contract::send_message(msg).unwrap();

        println!("message {} sent!", hex::encode(msg_id.as_slice()));
    }
}