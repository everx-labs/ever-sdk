use ton_abi_json::json_abi::decode_function_response;
use super::*;
use std::io::{Cursor};
use reql::{Config, Client, Run};
use serde_json::Value;
use reql_types::WriteStatus;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha512;
use tvm::types::AccountId;
use tvm::stack::{BuilderData, IBitstring};

const DB_NAME: &str = "blockchain";
const WORKCHAIN: i32 = 0;

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
                "id": msg_id.to_hex_string(),
                MSG_STATE_FIELD_NAME: MessageProcessingStatus::Queued
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

        for state in [MessageProcessingStatus::Processing, MessageProcessingStatus::Proposed, MessageProcessingStatus::Finalized].iter() {

            let insert_doc = r.db(DB_NAME)
                .table(MSG_TABLE_NAME)
                .replace(json!({
                    "id": msg_id_.to_hex_string(),
                    MSG_STATE_FIELD_NAME: state
                 }))
                .run::<WriteStatus>(conn).unwrap().wait().next().unwrap();
            println!("\n\n insert \n {:#?}", insert_doc);
        }
    });

    // chech all changes were got    
    let mut changes_stream = changes_stream.wait();
    for state in [MessageProcessingStatus::Processing, MessageProcessingStatus::Proposed, MessageProcessingStatus::Finalized].iter() {
        let ccs = ContractCallState {
            message_id: msg_id.clone(),
            message_state: state.clone(),
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
    init_json(Some(WORKCHAIN), config_json.into()).unwrap();


    let msg_id = MessageId::default();

    // subscribe changes
    let changes_stream = Contract::subscribe_updates(msg_id.clone()).unwrap();

    // another thread - write changes into DB though Kafka (emulate node activity)
    let msg_id_ = msg_id.clone();
    let another_thread = std::thread::spawn(move || {

        std::thread::sleep(std::time::Duration::from_secs(1));

        for state in [MessageProcessingStatus::Processing, MessageProcessingStatus::Proposed, MessageProcessingStatus::Finalized].iter() {
            let key = format!("\"{}\"", msg_id_.to_hex_string());
            
            let doc = json!({
                "message_id": msg_id_.to_hex_string(),
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
    for state in [MessageProcessingStatus::Processing, MessageProcessingStatus::Proposed, MessageProcessingStatus::Finalized].iter() {
        let ccs = ContractCallState {
            message_id: msg_id.clone(),
            message_state: state.clone(),
        };

        let json = serde_json::to_string(&ccs).unwrap();
        println!("CLIENT {}", json);

        assert_eq!(changes_stream.next().unwrap().unwrap(), ccs);
    }

    another_thread.join().unwrap();
}

const SUBSCRIBE_CONTRACT_ABI: &str = r#"
{
    "ABI version": 0,
    "functions": [{
        "name": "constructor",
        "inputs": [{"name": "wallet", "type": "bits256"}],
        "outputs": []
    }, {
        "name": "subscribe",
        "signed": true,
        "inputs": [
            {"name": "pubkey", "type": "bits256"},
            {"name": "to",     "type": "bits256"},
            {"name": "value",  "type": "duint"},
            {"name": "period", "type": "duint"}
        ],
        "outputs": [{"name": "subscriptionHash", "type": "bits256"}]
    }, {
        "name": "cancel",
        "signed": true,
        "inputs": [{"name": "subscriptionHash", "type": "bits256"}],
        "outputs": []
    }, {
        "name": "executeSubscription",
        "inputs": [
            {"name": "subscriptionHash","type": "bits256"},
            {"name": "signature",       "type": "bits256"}
        ],
        "outputs": []
    }, {
        "name": "getSubscription",
        "inputs": [{"name": "subscriptionHash","type": "bits256"}],
        "outputs": [
            {"name": "to", "type": "bits256"},
            {"name": "amount", "type": "duint"},
            {"name": "period", "type": "duint"},
            {"name": "status", "type": "uint8"}
        ]
    }]
}"#;

const SUBSCRIBE_PARAMS: &str = r#"
{
	"pubkey": "x0000000000000000000000000000000000000000000000000000000000000001",
	"to": "x0000000000000000000000000000000000000000000000000000000000000002",
	"value": 1234567890,
	"period": 1234567890
}"#;

const CONSTRUCTOR_PARAMS: &str = r#"
{
	"wallet": "x0000000000000000000000000000000000000000000000000000000000000001"
}"#;


fn test_call_contract(address: AccountId, key_pair: &Keypair) {

    let func = "subscribe".to_string();
    let input = SUBSCRIBE_PARAMS.to_string();
    let abi = SUBSCRIBE_CONTRACT_ABI.to_string();

    let contract = Contract::load(address.into())
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .expect("Error unwrap contract while loading Contract");

    // call needed method
    let changes_stream = Contract::call_json(contract.id().into(), func.clone(), input, abi.clone(), Some(&key_pair))
        .expect("Error calling contract method");

    // wait transaction id in message-status 
    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            println!("next state: {:?}", s);
            if s.message_state == MessageProcessingStatus::Finalized {
                tr_id = Some(s.message_id.clone());
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
        .expect("Error unwrap result while loading Transaction")
        .expect("Error unwrap returned Transaction");

    // take external outbound message from the transaction
    let out_msg = tr.load_out_messages()
        .expect("Error calling load out messages")
        .wait()
        .find(|msg| {
            msg.as_ref()
                .expect("error unwrap out message 1")
                .as_ref()
                    .expect("error unwrap out message 2")
                    .msg_type() == MessageType::ExternalOutbound
        })
            .expect("erro unwrap out message 2")
            .expect("erro unwrap out message 3")
            .expect("erro unwrap out message 4");

    // take body from the message
    let response = out_msg.body().expect("erro unwrap out message body").into();


    // decode the body by ABI
    let result = decode_function_response(abi, func, response)
        .expect("Error decoding result");

    println!("result:/n{}", result);


    // this way it is need:
    // 1. message status with transaction id or transaction object with in-message id
    // 2. transaction object with out messages ids
    // 3. message object with body

}

#[test]
fn test_deploy_and_call_contract() {
   
    let config_json = r#"
        {
            "db_config": {
                "servers": ["142.93.137.28:28015"],
                "db_name": "blockchain"
            },
            "kafka_config": {
                "servers": ["142.93.137.28:9092"],
                "topic": "requests",
                "ack_timeout": 1000
            }
        }"#;    
    init_json(Some(WORKCHAIN), config_json.into()).unwrap();
   
   
    // read image from file and construct ContractImage
    let mut state_init = std::fs::File::open("src/tests/contract.tvc").expect("Unable to open contract code file");

    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let contract_image = ContractImage::from_state_init_and_key(&mut state_init, &keypair.public).expect("Unable to parse contract code file");

    let account_id = contract_image.account_id();

    // before deploying contract need to transfer some funds to its address
    println!("Account ID to take some grams {}", account_id);
    let msg = create_external_transfer_funds_message(AccountId::from([0_u8; 32]), account_id.clone(), 100);
    Contract::send_message(msg).unwrap();


    // call deploy method
    let func = "constructor".to_string();
    let input = CONSTRUCTOR_PARAMS.to_string();
    let abi = SUBSCRIBE_CONTRACT_ABI.to_string();

    let changes_stream = Contract::deploy_json(func, input, abi, contract_image, Some(&keypair))
        .expect("Error deploying contract");

    // wait transaction id in message-status or 
    // wait message will done and find transaction with the message

    // wait transaction id in message-status 
    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            println!("next state: {:?}", s);
            if s.message_state == MessageProcessingStatus::Finalized {
                tr_id = Some(s.message_id.clone());
                break;
            }
        }
    }
    // contract constructor doesn't return any values so there are no output messages in transaction
    // so just check deployment transaction created
    let _tr_id = tr_id.expect("Error: no transaction id");

    test_call_contract(account_id, &keypair);
}

/*#[test]
fn test_send_empty_messages() {
    let id = AccountId::from([11; 32]);
    let contract = Contract { id, balance_grams: 0 };
    
    let config_json = r#"
    {
        "servers": ["builder.tonlabs.io:9092"],
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
        
        let msg = Contract::create_message(contract.id(), msg_body).unwrap();

        // send message by Kafka
        let msg_id = Contract::send_message(msg).unwrap();

        println!("message {} sent!", hex::encode(msg_id.as_slice()));
    }
}*/

#[test]
fn test_contract_image_from_file() {
    let mut state_init = std::fs::File::open("src/tests/contract.tvc").expect("Unable to open contract code file");

    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let contract_image = ContractImage::from_state_init_and_key(&mut state_init, &keypair.public).expect("Unable to parse contract code file");

    println!("Account ID {}", contract_image.account_id());
}

#[test]
fn test_deploy_empty_contract() {
    // init SDK
    let config_json = r#"
        {
            "db_config": {
                "servers": ["142.93.137.28:28015"],
                "db_name": "blockchain"
            },
            "kafka_config": {
                "servers": ["builder.tonlabs.io:9092"],
                "topic": "requests",
                "ack_timeout": 1000
            }
        }"#;    
    init_json(Some(WORKCHAIN), config_json.into()).unwrap();


    let mut csprng = OsRng::new().unwrap();

    let mut code_builder = BuilderData::new();
    code_builder.append_u32(csprng.next_u32()).expect("Unable to add u32");
    let code_slice = SliceData::from(code_builder);

    let mut data = Vec::new();
    BagOfCells::with_root(code_slice.clone()).write_to(&mut data, false).expect("Error serializing BOC");
                                        
    let mut data_cur = Cursor::new(data);
    
    let image = ContractImage::new(&mut data_cur, None, None).expect("Error creating ContractImage");
    let acc_id = image.account_id();



    let msg = create_external_transfer_funds_message(AccountId::from([0_u8; 32]), image.account_id(), 1000);
    Contract::send_message(msg).unwrap();

    Contract::load(acc_id.into())
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .expect("Error unwrap contract while loading Contract");
    println!("Contract got!!!");



    let changes_stream = Contract::deploy_no_constructor(image)
        .expect("Error deploying contract");

        // wait transaction id in message-status 
    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            println!("next state: {:?}", s);
            if s.message_state == MessageProcessingStatus::Finalized {
                tr_id = Some(s.message_id.clone());
                break;
            }
        }
    }
    // contract constructor doesn't return any values so there are no output messages in transaction
    // so just check deployment transaction created
    let _tr_id = tr_id.expect("Error: no transaction id");
    println!("Transaction got!!!");

}


use rand::{thread_rng, Rng};
use ton_block::{Message, MsgAddressExt, MsgAddressInt, InternalMessageHeader, Grams, 
    ExternalInboundMessageHeader, CurrencyCollection, Serializable};

// Create message "from wallet" to transfer some funds 
// from one account to another
fn create_external_transfer_funds_message(src: AccountId, dst: AccountId, value: u128) -> Message {
    
    let mut rng = thread_rng();
    let mut builder = BuilderData::new();
    builder.append_u64(rng.gen::<u64>()).unwrap();
    let mut msg = Message::with_ext_in_header(
        ExternalInboundMessageHeader {
            src: MsgAddressExt::with_extern(&builder).unwrap(),
            dst: MsgAddressInt::with_standart(None, 0, src.clone()).unwrap(),
            import_fee: Grams::default(),
        }
    );

    let mut balance = CurrencyCollection::default();
    balance.grams = Grams(value.into());

    let int_msg_hdr = InternalMessageHeader::with_addresses(
            MsgAddressInt::with_standart(None, 0, src).unwrap(),
            MsgAddressInt::with_standart(None, 0, dst).unwrap(),
            balance);

    msg.body = Some(int_msg_hdr.write_to_new_cell().unwrap().into());

    msg
}

#[test]
fn test_load_nonexistent_contract() {

        // init SDK
    let config_json = r#"
        {
            "db_config": {
                "servers": ["142.93.137.28:28015"],
                "db_name": "blockchain"
            },
            "kafka_config": {
                "servers": ["builder.tonlabs.io:9092"],
                "topic": "requests-1",
                "ack_timeout": 1000
            }
        }"#;    
    init_json(Some(WORKCHAIN), config_json.into()).unwrap();

    let c = Contract::load(AccountId::from([67, 68, 69, 31, 67, 68, 69, 31, 67, 68, 69, 31, 67, 68, 69, 31, 67, 68, 69, 31, 67, 68, 69, 31, 67, 68, 69, 31, 67, 68, 69, 31]).into())
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract");

    assert!(c.is_none());
}