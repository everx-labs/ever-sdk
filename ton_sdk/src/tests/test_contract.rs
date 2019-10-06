use ton_abi::json_abi::decode_function_response;
use super::*;
use std::io::{Cursor};
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha512;
use tvm::block::TransactionProcessingStatus;
use tvm::types::AccountId;
use tvm::stack::{BuilderData, IBitstring};
use tests_common::*;
 
/*
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
            id: msg_id.clone(),
            status: state.clone(),
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
connect.rethink.kcql=UPSERT INTO messages_statuses SELECT * FROM messages_statuses AUTOCREATE PK id

    */


    // init SDK
    let config_json = CONFIG_JSON.clone();    
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
                "id": msg_id_.to_hex_string(),
                MSG_STATE_FIELD_NAME: state
            }).to_string();
            
            requests_helper::send_message_to_topic(
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
            id: msg_id.clone(),
            status: state.clone(),
        };

        let json = serde_json::to_string(&ccs).unwrap();
        println!("CLIENT {}", json);

        assert_eq!(changes_stream.next().unwrap().unwrap(), ccs);
    }

    another_thread.join().unwrap();
}
*/

const SUBSCRIBE_PARAMS: &str = r#"
{
    "subscriptionId": "x0000000000000000000000000000000000000000000000000000000000000001",
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
    let abi = test_piggy_bank::SUBSCRIBE_CONTRACT_ABI.to_string();

    // call needed method
    let changes_stream = Contract::call_json(address.into(), func.clone(), input, abi.clone(), Some(&key_pair))
        .expect("Error calling contract method");

    // wait transaction id in message-status 
    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            println!("next state: {:?}", s);
            if s.status == TransactionProcessingStatus::Finalized {
                tr_id = Some(s.id.clone());
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
    let response = out_msg.body().expect("erro unwrap out message body");


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
   
    tests_common::init_node_connection();   
   
    // read image from file and construct ContractImage
    let mut state_init = std::fs::File::open("src/tests/Subscription.tvc").expect("Unable to open contract code file");

    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let contract_image = ContractImage::from_state_init_and_key(&mut state_init, &keypair.public).expect("Unable to parse contract code file");

    let account_id = contract_image.account_id();

    // before deploying contract need to transfer some funds to its address
    println!("Account ID to take some grams {}", account_id);
    
    tests_common::get_grams_from_giver(account_id.clone());


    // call deploy method
    let func = "constructor".to_string();
    let input = CONSTRUCTOR_PARAMS.to_string();
    let abi = test_piggy_bank::SUBSCRIBE_CONTRACT_ABI.to_string();

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
            if s.status == TransactionProcessingStatus::Finalized {
                tr_id = Some(s.id.clone());
                break;
            }
        }
    }
    // contract constructor doesn't return any values so there are no output messages in transaction
    // so just check deployment transaction created
    let _tr_id = tr_id.expect("Error: no transaction id");

    test_call_contract(account_id, &keypair);
}

#[test]
fn test_contract_image_from_file() {
    let mut state_init = std::fs::File::open("src/tests/Subscription.tvc").expect("Unable to open contract code file");

    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let contract_image = ContractImage::from_state_init_and_key(&mut state_init, &keypair.public).expect("Unable to parse contract code file");

    println!("Account ID {}", contract_image.account_id());
}

#[test]
fn test_deploy_empty_contract() {
    init_node_connection();

    let mut csprng = OsRng::new().unwrap();

    let mut code_builder = BuilderData::new();
    code_builder.append_u32(csprng.next_u32()).expect("Unable to add u32");

    let mut data = Vec::new();
    BagOfCells::with_root(&Arc::<CellData>::from(&code_builder)).write_to(&mut data, false).expect("Error serializing BOC");
                                        
    let mut data_cur = Cursor::new(data);
    
    let image = ContractImage::new(&mut data_cur, None, None).expect("Error creating ContractImage");
    let acc_id = image.account_id();

    tests_common::get_grams_from_giver(acc_id.clone());

    println!("Account ID {}", acc_id);

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
            if s.status == TransactionProcessingStatus::Finalized {
                tr_id = Some(s.id.clone());
                break;
            }
        }
    }
    // contract constructor doesn't return any values so there are no output messages in transaction
    // so just check deployment transaction created
    let _tr_id = tr_id.expect("Error: no transaction id");
    println!("Transaction got!!!");

}

#[test]
fn test_load_nonexistent_contract() {
    init_node_connection();

    let c = Contract::load(AccountId::from([67, 68, 69, 31, 67, 68, 69, 31, 67, 68, 69, 31, 67, 68, 69, 31, 67, 68, 69, 31, 67, 68, 69, 31, 67, 68, 69, 31, 67, 68, 69, 31]).into())
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract");

    assert!(c.is_none());
}

#[test]
fn test_address_parsing() {
    Contract::set_default_workchain(Some(-1));

    let short = "fcb91a3a3816d0f7b8c2c76108b8a9bc5a6b7a55bd79f8ab101c52db29232260";
    let full_std = "-1:fcb91a3a3816d0f7b8c2c76108b8a9bc5a6b7a55bd79f8ab101c52db29232260";
    let base64 = "kf/8uRo6OBbQ97jCx2EIuKm8Wmt6Vb15+KsQHFLbKSMiYIny";
    let base64_url = "kf_8uRo6OBbQ97jCx2EIuKm8Wmt6Vb15-KsQHFLbKSMiYIny";

    let address = tvm::block::MsgAddressInt::with_standart(None, -1, hex::decode(short).unwrap().into()).unwrap();

    assert_eq!(address, AccountAddress::from_str(short).expect("Couldn't parse short address").get_msg_address().unwrap());
    assert_eq!(address, AccountAddress::from_str(full_std).expect("Couldn't parse full_std address").get_msg_address().unwrap());
    assert_eq!(address, AccountAddress::from_str(base64).expect("Couldn't parse base64 address").get_msg_address().unwrap());
    assert_eq!(address, AccountAddress::from_str(base64_url).expect("Couldn't parse base64_url address").get_msg_address().unwrap());

    assert_eq!(AccountAddress::from(address.clone()).as_base64(true, true, false).unwrap(), base64);
    assert_eq!(AccountAddress::from(address).as_base64(true, true, true).unwrap(), base64_url);
}

#[test]
fn test_print_base64_address_from_hex() {
    let hex_address = "0:9f2bc8a81da52c6b8cb1878352120f21e254138fff0b897f44fb6ff2b8cae256";

    let address = AccountAddress::from_str(hex_address).unwrap();

    println!("{}", address.as_base64(false, false, false).unwrap());
}

#[test]
fn test_store_pubkey() {
    let mut test_map = HashmapE::with_bit_len(DATA_MAP_KEYLEN);
    let test_pubkey = vec![11u8; 32];
    test_map.set(
        0u64.write_to_new_cell().unwrap().into(),
        &BuilderData::with_raw(vec![0u8; 32], 256).unwrap().into(),
    ).unwrap();

    let mut data = BuilderData::new();
    data.append_bit_one().unwrap()
        .checked_append_reference(test_map.data().unwrap()).unwrap();

    let new_data = insert_pubkey(Some(data.into()), &test_pubkey).unwrap();

    let new_map = HashmapE::with_data(DATA_MAP_KEYLEN, new_data.into());
    let key_slice = new_map.get(
        0u64.write_to_new_cell().unwrap().into(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(key_slice.get_bytestring(0), test_pubkey);
}
